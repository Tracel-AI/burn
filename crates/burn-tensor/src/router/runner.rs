use alloc::sync::Arc;
use spin::Mutex;

use crate::{
    binary_float_cmp_ops, binary_float_ops, binary_int_cmp_ops, binary_int_ops,
    repr::{
        BaseOperationDescription, BoolOperationDescription, FloatOperationDescription,
        HandleContainer, IntOperationDescription, ModuleOperationDescription,
        NumericOperationDescription, OperationDescription, ReprBackend, TensorDescription,
        TensorId,
    },
    scalar_float2int_ops, scalar_float_cmp_ops, scalar_float_dim_ops, scalar_float_ops,
    scalar_int_cmp_ops, scalar_int_dim_ops, scalar_int_ops, unary_float_ops, unary_int_ops, DType,
    ElementConversion, Shape, TensorData,
};

use super::{RouterTensor, RunnerClient};

/// A runner's context contains a [handle container](HandleContainer) to manage
/// (i.e., fetch and update) existing tensors.
pub struct RunnerContext<B: ReprBackend> {
    /// Handle container to retrieve tensors based on their description.
    handles: HandleContainer<B::Handle>,
}

impl<B: ReprBackend> RunnerContext<B> {
    /// Create a new (uninitialized) empty tensor and returns its corresponding [tensor id](TensorId).
    fn create_empty_handle(&mut self) -> Arc<TensorId> {
        self.handles.create_tensor_uninit()
    }
}

/// A runner is responsible for executing tensor operations for a given [intermediate backend](ReprBackend).
#[derive(Clone)]
pub struct Runner<B: ReprBackend> {
    // Mutex for the mutable handles
    context: Arc<Mutex<RunnerContext<B>>>,
    device: B::Device,
}

impl<B: ReprBackend> Runner<B> {
    pub(crate) fn new(device: B::Device) -> Self {
        Self {
            context: Arc::new(Mutex::new(RunnerContext {
                handles: HandleContainer::new(),
            })),
            device,
        }
    }

    /// Get the tensor handle for the given [tensor description](TensorDescription).
    pub(crate) fn get_tensor_handle(&self, tensor: &TensorDescription) -> B::Handle {
        let handles = &mut self.context.lock().handles;
        handles.get_tensor_handle(tensor).handle
    }

    /// Create a tensor with the given handle and shape.
    pub(crate) fn register_tensor<C: RunnerClient>(
        &self,
        handle: B::Handle,
        shape: Vec<usize>,
        dtype: DType,
        client: C,
    ) -> RouterTensor<C> {
        let mut ctx = self.context.lock();
        let id = ctx.create_empty_handle();

        ctx.handles.register_handle(*id.as_ref(), handle);
        core::mem::drop(ctx);

        RouterTensor {
            id,
            shape,
            dtype,
            client,
        }
    }
}

impl<B: ReprBackend> RunnerClient for Runner<B> {
    type Device = B::Device;

    /// Execute a tensor operation.
    fn register(&self, op: OperationDescription) {
        match &op {
            // For every op: get the input(s), execute the operation and register the output(s)
            OperationDescription::BaseFloat(op) => match op {
                BaseOperationDescription::ToDevice(_) => unreachable!(),
                BaseOperationDescription::Reshape(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.input);

                    let output = B::float_reshape(tensor, desc.out.shape.clone().into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::SwapDims(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.input);

                    let output = B::float_swap_dims(tensor, desc.dim1, desc.dim2);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Permute(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.input);

                    let output = B::float_permute(tensor, &desc.axes);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Flip(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.input);

                    let output = B::float_flip(tensor, &desc.axes);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Expand(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.input);

                    let output = B::float_expand(tensor, desc.shape.clone().into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Slice(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);

                    let output = B::float_slice(tensor, &desc.ranges);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::SliceAssign(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);
                    let value = handles.get_float_tensor::<B>(&desc.value);

                    let output = B::float_slice_assign(tensor, &desc.ranges, value);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Equal(desc) => {
                    binary_float_cmp_ops!(self.context, desc, B::float_equal)
                }
                BaseOperationDescription::RepeatDim(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);

                    let output = B::float_repeat_dim(tensor, desc.dim, desc.times);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Cat(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensors = desc
                        .tensors
                        .iter()
                        .map(|tensor| handles.get_float_tensor::<B>(tensor))
                        .collect();

                    let output = B::float_cat(tensors, desc.dim);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Cast(_) => unreachable!(),
            },
            OperationDescription::BaseInt(op) => match op {
                BaseOperationDescription::ToDevice(_) => unreachable!(),
                BaseOperationDescription::Reshape(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.input);

                    let output = B::int_reshape(tensor, desc.out.shape.clone().into());
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::SwapDims(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.input);

                    let output = B::int_swap_dims(tensor, desc.dim1, desc.dim2);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Permute(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.input);

                    let output = B::int_permute(tensor, &desc.axes);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Flip(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.input);

                    let output = B::int_flip(tensor, &desc.axes);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Expand(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.input);

                    let output = B::int_expand(tensor, desc.shape.clone().into());
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Slice(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);

                    let output = B::int_slice(tensor, &desc.ranges);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::SliceAssign(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);
                    let value = handles.get_int_tensor::<B>(&desc.value);

                    let output = B::int_slice_assign(tensor, &desc.ranges, value);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Equal(desc) => {
                    binary_int_cmp_ops!(self.context, desc, B::int_equal)
                }
                BaseOperationDescription::RepeatDim(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);

                    let output = B::int_repeat_dim(tensor, desc.dim, desc.times);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Cat(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensors = desc
                        .tensors
                        .iter()
                        .map(|tensor| handles.get_int_tensor::<B>(tensor))
                        .collect();

                    let output = B::int_cat(tensors, desc.dim);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Cast(_) => unreachable!(),
            },
            OperationDescription::BaseBool(op) => match op {
                BaseOperationDescription::ToDevice(_) => unreachable!(),
                BaseOperationDescription::Reshape(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_reshape(tensor, desc.out.shape.clone().into());
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::SwapDims(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_swap_dims(tensor, desc.dim1, desc.dim2);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Permute(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_permute(tensor, &desc.axes);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Flip(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_flip(tensor, &desc.axes);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Expand(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_expand(tensor, desc.shape.clone().into());
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Slice(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.tensor);

                    let output = B::bool_slice(tensor, &desc.ranges);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::SliceAssign(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.tensor);
                    let value = handles.get_bool_tensor::<B>(&desc.value);

                    let output = B::bool_slice_assign(tensor, &desc.ranges, value);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Equal(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let lhs = handles.get_bool_tensor::<B>(&desc.lhs);
                    let rhs = handles.get_bool_tensor::<B>(&desc.rhs);

                    let output = B::bool_equal(lhs, rhs);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::RepeatDim(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.tensor);

                    let output = B::bool_repeat_dim(tensor, desc.dim, desc.times);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Cat(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensors = desc
                        .tensors
                        .iter()
                        .map(|tensor| handles.get_bool_tensor::<B>(tensor))
                        .collect();

                    let output = B::bool_cat(tensors, desc.dim);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
                BaseOperationDescription::Cast(_) => unreachable!(),
            },
            OperationDescription::NumericFloat(_dtype, op) => match op {
                NumericOperationDescription::Add(desc) => {
                    binary_float_ops!(self.context, desc, B::float_add)
                }
                NumericOperationDescription::AddScalar(desc) => {
                    scalar_float_ops!(self.context, desc, B::float_add_scalar)
                }
                NumericOperationDescription::Sub(desc) => {
                    binary_float_ops!(self.context, desc, B::float_sub)
                }
                NumericOperationDescription::SubScalar(desc) => {
                    scalar_float_ops!(self.context, desc, B::float_sub_scalar)
                }
                NumericOperationDescription::Div(desc) => {
                    binary_float_ops!(self.context, desc, B::float_div)
                }
                NumericOperationDescription::DivScalar(desc) => {
                    scalar_float_ops!(self.context, desc, B::float_div_scalar)
                }
                NumericOperationDescription::RemScalar(desc) => {
                    scalar_float_ops!(self.context, desc, B::float_remainder_scalar)
                }
                NumericOperationDescription::Mul(desc) => {
                    binary_float_ops!(self.context, desc, B::float_mul)
                }
                NumericOperationDescription::MulScalar(desc) => {
                    scalar_float_ops!(self.context, desc, B::float_mul_scalar)
                }
                NumericOperationDescription::Abs(desc) => {
                    unary_float_ops!(self.context, desc, B::float_abs)
                }
                NumericOperationDescription::Ones(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.shape.clone());
                    let output = B::float_ones(shape, &self.device);
                    handles.register_float_tensor::<B>(&desc.id, output);
                }
                NumericOperationDescription::Zeros(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.shape.clone());
                    let output = B::float_zeros(shape, &self.device);
                    handles.register_float_tensor::<B>(&desc.id, output);
                }
                NumericOperationDescription::Full((desc, elem)) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.shape.clone());
                    let output = B::float_full(shape, elem.elem(), &self.device);
                    handles.register_float_tensor::<B>(&desc.id, output);
                }
                NumericOperationDescription::Gather(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);

                    let output = B::float_gather(desc.dim, tensor, indices);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::Scatter(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);
                    let value = handles.get_float_tensor::<B>(&desc.value);

                    let output = B::float_scatter(desc.dim, tensor, indices, value);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::Select(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);

                    let output = B::float_select(tensor, desc.dim, indices);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::SelectAssign(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);
                    let value = handles.get_float_tensor::<B>(&desc.value);

                    let output = B::float_select_assign(tensor, desc.dim, indices, value);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::MaskWhere(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);
                    let mask = handles.get_bool_tensor::<B>(&desc.mask);
                    let value = handles.get_float_tensor::<B>(&desc.value);

                    let output = B::float_mask_where(tensor, mask, value);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::MaskFill(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);
                    let mask = handles.get_bool_tensor::<B>(&desc.mask);

                    let output = B::float_mask_fill(tensor, mask, desc.value.elem());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::MeanDim(desc) => {
                    scalar_float_dim_ops!(self.context, desc, B::float_mean_dim)
                }
                NumericOperationDescription::Mean(desc) => {
                    unary_float_ops!(self.context, desc, B::float_mean)
                }
                NumericOperationDescription::Sum(desc) => {
                    unary_float_ops!(self.context, desc, B::float_sum)
                }
                NumericOperationDescription::SumDim(desc) => {
                    scalar_float_dim_ops!(self.context, desc, B::float_sum_dim)
                }
                NumericOperationDescription::Prod(desc) => {
                    unary_float_ops!(self.context, desc, B::float_prod)
                }
                NumericOperationDescription::ProdDim(desc) => {
                    scalar_float_dim_ops!(self.context, desc, B::float_prod_dim)
                }
                NumericOperationDescription::EqualElem(desc) => {
                    scalar_float_cmp_ops!(self.context, desc, B::float_equal_elem)
                }
                NumericOperationDescription::Greater(desc) => {
                    binary_float_cmp_ops!(self.context, desc, B::float_greater)
                }
                NumericOperationDescription::GreaterElem(desc) => {
                    scalar_float_cmp_ops!(self.context, desc, B::float_greater_elem)
                }
                NumericOperationDescription::GreaterEqual(desc) => {
                    binary_float_cmp_ops!(self.context, desc, B::float_greater_equal)
                }
                NumericOperationDescription::GreaterEqualElem(desc) => {
                    scalar_float_cmp_ops!(self.context, desc, B::float_greater_equal_elem)
                }
                NumericOperationDescription::Lower(desc) => {
                    binary_float_cmp_ops!(self.context, desc, B::float_lower)
                }
                NumericOperationDescription::LowerElem(desc) => {
                    scalar_float_cmp_ops!(self.context, desc, B::float_lower_elem)
                }
                NumericOperationDescription::LowerEqual(desc) => {
                    binary_float_cmp_ops!(self.context, desc, B::float_lower_equal)
                }
                NumericOperationDescription::LowerEqualElem(desc) => {
                    scalar_float_cmp_ops!(self.context, desc, B::float_lower_equal_elem)
                }
                NumericOperationDescription::ArgMax(desc) => {
                    scalar_float2int_ops!(self.context, desc, B::float_argmax)
                }
                NumericOperationDescription::ArgMin(desc) => {
                    scalar_float2int_ops!(self.context, desc, B::float_argmin)
                }
                NumericOperationDescription::Max(desc) => {
                    unary_float_ops!(self.context, desc, B::float_max)
                }
                NumericOperationDescription::MaxDimWithIndices(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);

                    let (output, output_idx) = B::float_max_dim_with_indices(tensor, desc.dim);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                    handles.register_int_tensor::<B>(&desc.out_indices.id, output_idx);
                }
                NumericOperationDescription::MinDimWithIndices(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);

                    let (output, output_idx) = B::float_min_dim_with_indices(tensor, desc.dim);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                    handles.register_int_tensor::<B>(&desc.out_indices.id, output_idx);
                }
                NumericOperationDescription::Min(desc) => {
                    unary_float_ops!(self.context, desc, B::float_min)
                }
                NumericOperationDescription::MaxDim(desc) => {
                    scalar_float_dim_ops!(self.context, desc, B::float_max_dim)
                }
                NumericOperationDescription::MinDim(desc) => {
                    scalar_float_dim_ops!(self.context, desc, B::float_min_dim)
                }
                NumericOperationDescription::Clamp(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.tensor);

                    let output = B::float_clamp(tensor, desc.min.elem(), desc.max.elem());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::IntRandom(_) => unreachable!(),
                NumericOperationDescription::Powf(desc) => {
                    binary_float_ops!(self.context, desc, B::float_powf)
                }
            },
            OperationDescription::NumericInt(_dtype, op) => match op {
                NumericOperationDescription::Add(desc) => {
                    binary_int_ops!(self.context, desc, B::int_add)
                }
                NumericOperationDescription::AddScalar(desc) => {
                    scalar_int_ops!(self.context, desc, B::int_add_scalar)
                }
                NumericOperationDescription::Sub(desc) => {
                    binary_int_ops!(self.context, desc, B::int_sub)
                }
                NumericOperationDescription::SubScalar(desc) => {
                    scalar_int_ops!(self.context, desc, B::int_sub_scalar)
                }
                NumericOperationDescription::Div(desc) => {
                    binary_int_ops!(self.context, desc, B::int_div)
                }
                NumericOperationDescription::DivScalar(desc) => {
                    scalar_int_ops!(self.context, desc, B::int_div_scalar)
                }
                NumericOperationDescription::RemScalar(desc) => {
                    scalar_int_ops!(self.context, desc, B::int_remainder_scalar)
                }
                NumericOperationDescription::Mul(desc) => {
                    binary_int_ops!(self.context, desc, B::int_mul)
                }
                NumericOperationDescription::MulScalar(desc) => {
                    scalar_int_ops!(self.context, desc, B::int_mul_scalar)
                }
                NumericOperationDescription::Abs(desc) => {
                    unary_int_ops!(self.context, desc, B::int_abs)
                }
                NumericOperationDescription::Ones(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.shape.clone());
                    let output = B::int_ones(shape, &self.device);
                    handles.register_int_tensor::<B>(&desc.id, output);
                }
                NumericOperationDescription::Zeros(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.shape.clone());
                    let output = B::int_zeros(shape, &self.device);
                    handles.register_int_tensor::<B>(&desc.id, output);
                }
                NumericOperationDescription::Full((desc, elem)) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.shape.clone());
                    let output = B::int_full(shape, elem.elem(), &self.device);
                    handles.register_int_tensor::<B>(&desc.id, output);
                }
                NumericOperationDescription::Gather(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);

                    let output = B::int_gather(desc.dim, tensor, indices);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::Scatter(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);
                    let value = handles.get_int_tensor::<B>(&desc.value);

                    let output = B::int_scatter(desc.dim, tensor, indices, value);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::Select(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);

                    let output = B::int_select(tensor, desc.dim, indices);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::SelectAssign(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);
                    let value = handles.get_int_tensor::<B>(&desc.value);

                    let output = B::int_select_assign(tensor, desc.dim, indices, value);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::MaskWhere(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);
                    let mask = handles.get_bool_tensor::<B>(&desc.mask);
                    let value = handles.get_int_tensor::<B>(&desc.value);

                    let output = B::int_mask_where(tensor, mask, value);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::MaskFill(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);
                    let mask = handles.get_bool_tensor::<B>(&desc.mask);

                    let output = B::int_mask_fill(tensor, mask, desc.value.elem());
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::MeanDim(desc) => {
                    scalar_int_dim_ops!(self.context, desc, B::int_mean_dim)
                }
                NumericOperationDescription::Mean(desc) => {
                    unary_int_ops!(self.context, desc, B::int_mean)
                }
                NumericOperationDescription::Sum(desc) => {
                    unary_int_ops!(self.context, desc, B::int_sum)
                }
                NumericOperationDescription::SumDim(desc) => {
                    scalar_int_dim_ops!(self.context, desc, B::int_sum_dim)
                }
                NumericOperationDescription::Prod(desc) => {
                    unary_int_ops!(self.context, desc, B::int_prod)
                }
                NumericOperationDescription::ProdDim(desc) => {
                    scalar_int_dim_ops!(self.context, desc, B::int_prod_dim)
                }
                NumericOperationDescription::EqualElem(desc) => {
                    scalar_int_cmp_ops!(self.context, desc, B::int_equal_elem)
                }
                NumericOperationDescription::Greater(desc) => {
                    binary_int_cmp_ops!(self.context, desc, B::int_greater)
                }
                NumericOperationDescription::GreaterElem(desc) => {
                    scalar_int_cmp_ops!(self.context, desc, B::int_greater_elem)
                }
                NumericOperationDescription::GreaterEqual(desc) => {
                    binary_int_cmp_ops!(self.context, desc, B::int_greater_equal)
                }
                NumericOperationDescription::GreaterEqualElem(desc) => {
                    scalar_int_cmp_ops!(self.context, desc, B::int_greater_equal_elem)
                }
                NumericOperationDescription::Lower(desc) => {
                    binary_int_cmp_ops!(self.context, desc, B::int_lower)
                }
                NumericOperationDescription::LowerElem(desc) => {
                    scalar_int_cmp_ops!(self.context, desc, B::int_lower_elem)
                }
                NumericOperationDescription::LowerEqual(desc) => {
                    binary_int_cmp_ops!(self.context, desc, B::int_lower_equal)
                }
                NumericOperationDescription::LowerEqualElem(desc) => {
                    scalar_int_cmp_ops!(self.context, desc, B::int_lower_equal_elem)
                }
                NumericOperationDescription::ArgMax(desc) => {
                    scalar_int_dim_ops!(self.context, desc, B::int_argmax)
                }
                NumericOperationDescription::ArgMin(desc) => {
                    scalar_int_dim_ops!(self.context, desc, B::int_argmin)
                }
                NumericOperationDescription::Max(desc) => {
                    unary_int_ops!(self.context, desc, B::int_max)
                }
                NumericOperationDescription::MaxDimWithIndices(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);

                    let (output, output_idx) = B::int_max_dim_with_indices(tensor, desc.dim);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                    handles.register_int_tensor::<B>(&desc.out_indices.id, output_idx);
                }
                NumericOperationDescription::MinDimWithIndices(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);

                    let (output, output_idx) = B::int_min_dim_with_indices(tensor, desc.dim);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                    handles.register_int_tensor::<B>(&desc.out_indices.id, output_idx);
                }
                NumericOperationDescription::Min(desc) => {
                    unary_int_ops!(self.context, desc, B::int_min)
                }
                NumericOperationDescription::MaxDim(desc) => {
                    scalar_int_dim_ops!(self.context, desc, B::int_max_dim)
                }
                NumericOperationDescription::MinDim(desc) => {
                    scalar_int_dim_ops!(self.context, desc, B::int_min_dim)
                }
                NumericOperationDescription::Clamp(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.tensor);

                    let output = B::int_clamp(tensor, desc.min.elem(), desc.max.elem());
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::IntRandom(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.out.shape.clone());

                    let output = B::int_random(shape, desc.distribution, &self.device);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                NumericOperationDescription::Powf(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let lhs = handles.get_int_tensor::<B>(&desc.lhs);
                    let rhs = handles.get_float_tensor::<B>(&desc.rhs);

                    let output = B::int_powf(lhs, rhs);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
            },
            OperationDescription::Bool(op) => match op {
                BoolOperationDescription::IntoFloat(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_into_float(tensor);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                BoolOperationDescription::IntoInt(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_into_int(tensor);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                BoolOperationDescription::Not(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_bool_tensor::<B>(&desc.input);

                    let output = B::bool_not(tensor);
                    handles.register_bool_tensor::<B>(&desc.out.id, output);
                }
            },
            OperationDescription::Int(op) => match op {
                IntOperationDescription::IntoFloat(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_int_tensor::<B>(&desc.input);

                    let output = B::int_into_float(tensor);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
            },
            OperationDescription::Float(_dtype, op) => match op {
                FloatOperationDescription::Exp(desc) => {
                    unary_float_ops!(self.context, desc, B::float_exp)
                }
                FloatOperationDescription::Log(desc) => {
                    unary_float_ops!(self.context, desc, B::float_log)
                }
                FloatOperationDescription::Log1p(desc) => {
                    unary_float_ops!(self.context, desc, B::float_log1p)
                }
                FloatOperationDescription::Erf(desc) => {
                    unary_float_ops!(self.context, desc, B::float_erf)
                }
                FloatOperationDescription::PowfScalar(desc) => {
                    scalar_float_ops!(self.context, desc, B::float_powf_scalar)
                }
                FloatOperationDescription::Sqrt(desc) => {
                    unary_float_ops!(self.context, desc, B::float_sqrt)
                }
                FloatOperationDescription::Cos(desc) => {
                    unary_float_ops!(self.context, desc, B::float_cos)
                }
                FloatOperationDescription::Sin(desc) => {
                    unary_float_ops!(self.context, desc, B::float_sin)
                }
                FloatOperationDescription::Tanh(desc) => {
                    unary_float_ops!(self.context, desc, B::float_sin)
                }
                FloatOperationDescription::IntoInt(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let tensor = handles.get_float_tensor::<B>(&desc.input);

                    let output = B::float_into_int(tensor);
                    handles.register_int_tensor::<B>(&desc.out.id, output);
                }
                FloatOperationDescription::Matmul(desc) => {
                    binary_float_ops!(self.context, desc, B::float_matmul)
                }
                FloatOperationDescription::Random(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let shape = Shape::from(desc.out.shape.clone());

                    let output = B::float_random(shape, desc.distribution, &self.device);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                FloatOperationDescription::Recip(desc) => {
                    unary_float_ops!(self.context, desc, B::float_recip)
                }
                FloatOperationDescription::Quantize(_) => todo!(),
                FloatOperationDescription::Dequantize(_) => todo!(),
            },
            OperationDescription::Module(op) => match op {
                ModuleOperationDescription::Embedding(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let weights = handles.get_float_tensor::<B>(&desc.weights);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);

                    let output = B::embedding(weights, indices);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::EmbeddingBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let weights = handles.get_float_tensor::<B>(&desc.weights);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);
                    let output_grad = handles.get_float_tensor::<B>(&desc.out_grad);

                    let output = B::embedding_backward(weights, output_grad, indices);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::Conv1d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));

                    let output = B::conv1d(x, weight, bias, desc.clone().options.into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::Conv2d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));

                    let output = B::conv2d(x, weight, bias, desc.clone().options.into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::Conv3d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));

                    let output = B::conv3d(x, weight, bias, desc.options.clone().into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::DeformableConv2d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let offset = handles.get_float_tensor::<B>(&desc.offset);
                    let mask = desc
                        .mask
                        .as_ref()
                        .map(|mask| handles.get_float_tensor::<B>(mask));
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));

                    let output = B::deform_conv2d(
                        x,
                        offset,
                        weight,
                        mask,
                        bias,
                        desc.options.clone().into(),
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::DeformableConv2dBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let offset = handles.get_float_tensor::<B>(&desc.offset);
                    let mask = desc
                        .mask
                        .as_ref()
                        .map(|mask| handles.get_float_tensor::<B>(mask));
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));
                    let output_grad = handles.get_float_tensor::<B>(&desc.out_grad);

                    let output = B::deform_conv2d_backward(
                        x,
                        offset,
                        weight,
                        mask,
                        bias,
                        output_grad,
                        desc.options.clone().into(),
                    );

                    handles.register_float_tensor::<B>(&desc.input_grad.id, output.x_grad);
                    handles.register_float_tensor::<B>(&desc.offset_grad.id, output.offset_grad);
                    handles.register_float_tensor::<B>(&desc.weight_grad.id, output.weight_grad);
                    if let Some((mask_grad, field)) = output.mask_grad.zip(desc.mask_grad.as_ref())
                    {
                        handles.register_float_tensor::<B>(&field.id, mask_grad);
                    }
                    if let Some((bias_grad, field)) = output.bias_grad.zip(desc.bias_grad.as_ref())
                    {
                        handles.register_float_tensor::<B>(&field.id, bias_grad);
                    }
                }
                ModuleOperationDescription::ConvTranspose1d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));

                    let output = B::conv_transpose1d(x, weight, bias, desc.options.clone().into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::ConvTranspose2d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));

                    let output = B::conv_transpose2d(x, weight, bias, desc.options.clone().into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::ConvTranspose3d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let weight = handles.get_float_tensor::<B>(&desc.weight);
                    let bias = desc
                        .bias
                        .as_ref()
                        .map(|bias| handles.get_float_tensor::<B>(bias));

                    let output = B::conv_transpose3d(x, weight, bias, desc.options.clone().into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AvgPool1d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::avg_pool1d(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.count_include_pad,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AvgPool2d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::avg_pool2d(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.count_include_pad,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AvgPool1dBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let grad = handles.get_float_tensor::<B>(&desc.grad);

                    let output = B::avg_pool1d_backward(
                        x,
                        grad,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.count_include_pad,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AvgPool2dBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let grad = handles.get_float_tensor::<B>(&desc.grad);

                    let output = B::avg_pool2d_backward(
                        x,
                        grad,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.count_include_pad,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AdaptiveAvgPool1d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::adaptive_avg_pool1d(x, desc.output_size);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AdaptiveAvgPool2d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::adaptive_avg_pool2d(x, desc.output_size);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AdaptiveAvgPool1dBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let grad = handles.get_float_tensor::<B>(&desc.grad);

                    let output = B::adaptive_avg_pool1d_backward(x, grad);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::AdaptiveAvgPool2dBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let grad = handles.get_float_tensor::<B>(&desc.grad);

                    let output = B::adaptive_avg_pool2d_backward(x, grad);
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::MaxPool1d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::max_pool1d(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.dilation,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::MaxPool1dWithIndices(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::max_pool1d_with_indices(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.dilation,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output.output);
                    handles.register_int_tensor::<B>(&desc.out.id, output.indices);
                }
                ModuleOperationDescription::MaxPool1dWithIndicesBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let output_grad = handles.get_float_tensor::<B>(&desc.grad);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);

                    let output = B::max_pool1d_with_indices_backward(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.dilation,
                        output_grad,
                        indices,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output.x_grad);
                }
                ModuleOperationDescription::MaxPool2d(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::max_pool2d(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.dilation,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::MaxPool2dWithIndices(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::max_pool2d_with_indices(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.dilation,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output.output);
                    handles.register_int_tensor::<B>(&desc.out.id, output.indices);
                }
                ModuleOperationDescription::MaxPool2dWithIndicesBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let output_grad = handles.get_float_tensor::<B>(&desc.grad);
                    let indices = handles.get_int_tensor::<B>(&desc.indices);

                    let output = B::max_pool2d_with_indices_backward(
                        x,
                        desc.kernel_size,
                        desc.stride,
                        desc.padding,
                        desc.dilation,
                        output_grad,
                        indices,
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output.x_grad);
                }
                ModuleOperationDescription::Interpolate(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);

                    let output = B::interpolate(x, desc.output_size, desc.options.clone().into());
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
                ModuleOperationDescription::InterpolateBackward(desc) => {
                    let handles = &mut self.context.lock().handles;
                    let x = handles.get_float_tensor::<B>(&desc.x);
                    let grad = handles.get_float_tensor::<B>(&desc.grad);

                    let output = B::interpolate_backward(
                        x,
                        grad,
                        desc.output_size,
                        desc.options.clone().into(),
                    );
                    handles.register_float_tensor::<B>(&desc.out.id, output);
                }
            },
        }

        // Remove unused tensor handles
        // NOTE: only ReadWrite handles are removed
        let mut ctx = self.context.lock();
        op.nodes()
            .into_iter()
            .for_each(|tensor| ctx.handles.free(tensor));
    }

    async fn read_tensor(&self, tensor: TensorDescription) -> TensorData {
        let mut ctx = self.context.lock();
        let tensor = ctx.handles.get_float_tensor::<B>(&tensor);

        B::float_into_data(tensor).await
    }

    fn register_tensor_data(&self, data: TensorData) -> RouterTensor<Self> {
        let mut ctx = self.context.lock();
        let id = ctx.create_empty_handle();
        let shape = data.shape.clone();
        let dtype = data.dtype;

        if dtype.is_float() {
            let tensor = B::float_from_data(data, &self.device);
            ctx.handles.register_float_tensor::<B>(&id, tensor)
        } else if dtype.is_int() {
            let tensor = B::int_from_data(data, &self.device);
            ctx.handles.register_int_tensor::<B>(&id, tensor)
        } else if dtype.is_bool() {
            let tensor = B::bool_from_data(data, &self.device);
            ctx.handles.register_bool_tensor::<B>(&id, tensor)
        } else if let DType::QFloat(_) = dtype {
            todo!();
        }

        core::mem::drop(ctx);

        RouterTensor {
            id,
            shape,
            dtype,
            client: self.clone(),
        }
    }

    fn register_empty_tensor(&self, shape: Vec<usize>, dtype: DType) -> RouterTensor<Self> {
        let mut ctx = self.context.lock();
        let id = ctx.create_empty_handle();
        core::mem::drop(ctx);

        RouterTensor {
            id,
            shape,
            dtype,
            client: self.clone(),
        }
    }

    fn device(&self) -> Self::Device {
        self.device.clone()
    }
}
