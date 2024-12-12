use std::any::TypeId;
use std::marker::PhantomData;

use crate::fusion::elemwise::optimization::ElemwiseRunner;
use crate::fusion::on_write::ir::ElemwisePrecision;
use crate::kernel::matmul;
use crate::{fusion::JitFusionHandle, JitRuntime};
use crate::{BoolElement, FloatElement};
use burn_fusion::stream::Context;
use burn_tensor::repr::{BinaryOperationDescription, TensorDescription, TensorStatus};
use burn_tensor::Shape;
use cubecl::linalg::matmul::components;
use cubecl::linalg::matmul::components::MatmulProblem;
use cubecl::linalg::matmul::kernels::matmul::{CmmaSelector, PlaneMmaSelector};
use cubecl::linalg::matmul::kernels::{MatmulAvailabilityError, MatmulLaunchError};
use cubecl::linalg::tensor::{matrix_layout, MatrixLayout};
use cubecl::{client::ComputeClient, prelude::*};
use half::{bf16, f16};
use serde::{Deserialize, Serialize};

use crate::fusion::on_write::{
    ir::{Arg, ElemwiseConfig, GlobalArgsLaunch},
    trace::{FuseOnWriteTrace, TraceRunner},
};

use super::spec::FusedMatmulSpec;
use super::FusedMatmulInputLaunch;

#[derive(new)]
/// Fuse matmul operation followed by elemwise operations into a single kernel.
pub struct MatmulOptimization<R: JitRuntime> {
    trace: FuseOnWriteTrace,
    trace_fallback: FuseOnWriteTrace,
    client: ComputeClient<R::Server, R::Channel>,
    device: R::Device,
    len: usize,
    args: ((Arg, Arg, Arg), BinaryOperationDescription),
}

#[derive(Serialize, Deserialize, Debug)]
/// State for the [matrix optimization](MatmulOptimizationState).
pub struct MatmulOptimizationState {
    trace: FuseOnWriteTrace,
    trace_fallback: FuseOnWriteTrace,
    args: ((Arg, Arg, Arg), BinaryOperationDescription),
    len: usize,
}

impl<R: JitRuntime> MatmulOptimization<R> {
    /// Execute the optimization.
    pub fn execute<BT: BoolElement>(&mut self, context: &mut Context<'_, JitFusionHandle<R>>) {
        match self.trace.run::<R, BT, FusedMatmul<R>>(
            &self.client,
            &self.device,
            context,
            &FusedMatmul::new(self.args.0 .0, self.args.0 .1, self.args.0 .2),
        ) {
            Ok(_) => {}
            Err(_err) => match self.args.0 .0.precision() {
                ElemwisePrecision::F32 => self.execute_fallback::<BT, f32>(context),
                ElemwisePrecision::F16 => self.execute_fallback::<BT, f16>(context),
                ElemwisePrecision::BF16 => self.execute_fallback::<BT, bf16>(context),
                _ => panic!("Unsupported precision"),
            },
        }
    }

    /// Number of operations fused.
    pub fn num_ops_fused(&self) -> usize {
        self.len
    }

    /// Create an optimization from its [state](MatmulOptimizationState).
    pub fn from_state(device: &R::Device, state: MatmulOptimizationState) -> Self {
        Self {
            trace: state.trace,
            trace_fallback: state.trace_fallback,
            len: state.len,
            client: R::client(device),
            device: device.clone(),
            args: state.args.clone(),
        }
    }

    /// Convert the optimization to its [state](MatmulOptimizationState).
    pub fn to_state(&self) -> MatmulOptimizationState {
        MatmulOptimizationState {
            trace: self.trace.clone(),
            trace_fallback: self.trace_fallback.clone(),
            args: self.args.clone(),
            len: self.len,
        }
    }

    fn execute_fallback<BT: BoolElement, EG: FloatElement>(
        &mut self,
        context: &mut Context<'_, JitFusionHandle<R>>,
    ) {
        let (out_tensor, out_desc) = {
            let lhs = context.tensors.get(&self.args.1.lhs.id).unwrap().clone();
            let rhs = context.tensors.get(&self.args.1.rhs.id).unwrap().clone();

            let lhs_handle = context.handles.get_handle(&lhs.id, &self.args.1.lhs.status);
            let rhs_handle = context.handles.get_handle(&rhs.id, &self.args.1.rhs.status);

            let lhs_tensor = lhs_handle.into_tensor(Shape {
                dims: lhs.shape.clone(),
            });
            let rhs_tensor = rhs_handle.into_tensor(Shape {
                dims: rhs.shape.clone(),
            });
            let out_tensor = matmul::matmul::<R, EG>(
                lhs_tensor,
                rhs_tensor,
                None,
                matmul::MatmulStrategy::default(),
            );
            let out = context.handles.create_tensor_uninit();
            let out_desc = TensorDescription {
                id: *out,
                shape: out_tensor.shape.dims.clone(),
                status: TensorStatus::ReadOnly, // TODO Not correct but safer
                dtype: self.args.1.out.dtype.clone(),
            };
            (out_tensor, out_desc)
        };
        context
            .handles
            .register_handle(out_desc.id, JitFusionHandle::from(out_tensor));
        context.tensors.insert(self.args.1.out.id, out_desc);

        self.trace_fallback
            .run::<R, BT, ElemwiseRunner>(&self.client, &self.device, context, &ElemwiseRunner)
            .unwrap();
    }
}

#[derive(new)]
struct FusedMatmul<R: JitRuntime> {
    lhs: Arg,
    rhs: Arg,
    out: Arg,
    _runtime: PhantomData<R>,
}

impl<R: JitRuntime> TraceRunner<R> for FusedMatmul<R> {
    type Error = MatmulLaunchError;

    fn run<'a>(
        &'a self,
        client: &'a ComputeClient<R::Server, R::Channel>,
        inputs: GlobalArgsLaunch<'a, R>,
        outputs: GlobalArgsLaunch<'a, R>,
        config: &'a ElemwiseConfig,
    ) -> Result<(), MatmulLaunchError> {
        match self.out.precision() {
            ElemwisePrecision::F32 => self.matmul_fused::<f32>(client, inputs, outputs, config),
            ElemwisePrecision::F16 => self.matmul_fused::<f16>(client, inputs, outputs, config),
            ElemwisePrecision::BF16 => self.matmul_fused::<bf16>(client, inputs, outputs, config),
            _ => panic!("Unsupported precision"),
        }
    }
}

impl<R: JitRuntime> FusedMatmul<R> {
    fn matmul_fused<'a, EG: Numeric>(
        &'a self,
        client: &'a ComputeClient<R::Server, R::Channel>,
        inputs: GlobalArgsLaunch<'a, R>,
        outputs: GlobalArgsLaunch<'a, R>,
        config: &'a ElemwiseConfig,
    ) -> Result<(), MatmulLaunchError> {
        let disable_cmma: bool = false;

        let lhs_shape = inputs.shape(&self.lhs);
        let rhs_shape = inputs.shape(&self.rhs);

        let lhs_strides = inputs.strides(&self.lhs);
        let rhs_strides = inputs.strides(&self.rhs);

        let check_layout = |strides| match matrix_layout(strides) {
            MatrixLayout::Contiguous => (false, false),
            MatrixLayout::MildlyPermuted {
                transposed,
                batch_swap: _,
            } => (false, transposed),
            MatrixLayout::HighlyPermuted => (true, false),
        };

        let (lhs_make_contiguous, lhs_transposed) = check_layout(lhs_strides);
        let (rhs_make_contiguous, rhs_transposed) = check_layout(rhs_strides);

        if lhs_make_contiguous || rhs_make_contiguous {
            panic!("Unsupported");
        }

        let rank = lhs_shape.len();

        let m = lhs_shape[rank - 2] as u32;
        let k = lhs_shape[rank - 1] as u32;
        let n = rhs_shape[rank - 1] as u32;

        let lhs_line_size = inputs.line_size(&self.lhs);
        let rhs_line_size = inputs.line_size(&self.lhs);
        let out_line_size = outputs.line_size(&config.ref_layout);

        let problem = MatmulProblem {
            m: m as usize,
            n: n as usize,
            k: k as usize,
            batches: (
                lhs_shape[..lhs_shape.len() - 2].to_vec(),
                rhs_shape[..rhs_shape.len() - 2].to_vec(),
            ),
            lhs_layout: match lhs_transposed {
                true => components::MatrixLayout::ColMajor,
                false => components::MatrixLayout::RowMajor,
            },
            rhs_layout: match rhs_transposed {
                true => components::MatrixLayout::ColMajor,
                false => components::MatrixLayout::RowMajor,
            },
            lhs_line_size,
            rhs_line_size,
            out_line_size,
        };

        let plane_size = client
            .properties()
            .hardware_properties()
            .defined_plane_size();

        match plane_size {
            Some(32) => matmul_launch_kernel::<32, R, EG>(
                client,
                FusedMatmulInputLaunch::new(inputs, config, &self.lhs, &self.rhs, &self.out),
                outputs,
                disable_cmma,
                problem,
            ),
            Some(64) => matmul_launch_kernel::<64, R, EG>(
                client,
                FusedMatmulInputLaunch::new(inputs, config, &self.lhs, &self.rhs, &self.out),
                outputs,
                disable_cmma,
                problem,
            ),
            Some(plane_dim) => Err(MatmulLaunchError::Unavailable(
                MatmulAvailabilityError::PlaneDimUnsupported { plane_dim },
            )),
            None => Err(MatmulLaunchError::Unavailable(
                MatmulAvailabilityError::PlaneDimUnknown,
            )),
        }
    }
}

fn matmul_launch_kernel<'a, const PLANE_DIM: u32, R: Runtime, EG: Numeric>(
    client: &ComputeClient<R::Server, R::Channel>,
    input: FusedMatmulInputLaunch<'a, R>,
    output: GlobalArgsLaunch<'a, R>,
    disable_cmma: bool,
    problem: MatmulProblem,
) -> Result<(), MatmulLaunchError> {
    if disable_cmma {
        PlaneMmaSelector::select_kernel::<FusedMatmulSpec<{ PLANE_DIM }, EG, EG, f32>, R>(
            client, input, output, problem,
        )
    } else if TypeId::of::<EG>() == TypeId::of::<f16>()
        || TypeId::of::<EG>() == TypeId::of::<flex32>()
    {
        CmmaSelector::select_kernel::<FusedMatmulSpec<{ PLANE_DIM }, EG, f16, f32>, R>(
            client, input, output, problem,
        )
    } else {
        CmmaSelector::select_kernel::<FusedMatmulSpec<{ PLANE_DIM }, EG, tf32, f32>, R>(
            client, input, output, problem,
        )
    }
}
