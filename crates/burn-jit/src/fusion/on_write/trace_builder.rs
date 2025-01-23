use super::{
    ir::{Arg, BinaryElemwiseArgs, ElemwiseOp, ElemwisePrecision, LayoutInfo, UnaryElemwiseArgs},
    trace::{FuseOnWriteTrace, RegisteredTensors},
};
use burn_tensor::{
    repr::{TensorDescription, TensorId, TensorStatus},
    DType, Element,
};
use cubecl::prelude::Sequence;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct FuseOnWriteTraceBuilder {
    locals: Locals,
    outputs: RegisteredTensors,
    inputs: RegisteredTensors,
    scalars: BTreeMap<ElemwisePrecision, u32>,
    shapes: Vec<TensorDescription>,
    ops: Vec<ElemwiseOp>,
    reads: BTreeMap<TensorId, ElemwiseOp>,
    pub bool_precision: ElemwisePrecision,
    outputs_unhandled: Vec<Arg>,
    inputs_unhandled: Vec<TensorId>,
}

impl FuseOnWriteTraceBuilder {
    pub fn new(bool_precision: ElemwisePrecision) -> Self {
        Self {
            locals: Locals::default(),
            outputs: RegisteredTensors::default(),
            inputs: RegisteredTensors::default(),
            scalars: BTreeMap::default(),
            shapes: Vec::default(),
            ops: Vec::new(),
            reads: BTreeMap::new(),
            bool_precision,
            outputs_unhandled: Vec::new(),
            inputs_unhandled: Vec::new(),
        }
    }

    pub fn register_operation(&mut self, op: ElemwiseOp) {
        self.ops.push(op);
    }

    // Estimate how many bindings are in use right now. This can return more than the actual number
    // but should never return less.
    pub fn estimate_bindings(&self) -> u32 {
        let meta = 1;
        let inputs = self.inputs.len() as u32;
        let outputs = self.output_tensors().len() as u32;
        // In the future, scalars could be packed into 1 buffer or into the metadata, but currently take up
        // one slot per scalar.
        let scalar = self.scalars.len() as u32;
        meta + inputs + outputs + scalar
    }

    pub fn output_unhandled(&mut self, tensor: &TensorDescription) -> Arg {
        let arg = self.output(tensor);
        self.outputs_unhandled.push(arg.clone());
        arg
    }

    pub fn input_unhandled(&mut self, tensor: &TensorDescription) -> Arg {
        let precision = tensor.dtype.into();

        // Bool tensors are encoded as bool_precision.
        let precision_input = match precision {
            ElemwisePrecision::Bool => self.bool_precision,
            _ => precision,
        };
        let new_input = self.inputs.insert(precision_input, tensor.clone());
        let arg = Arg::Input(new_input, precision_input, LayoutInfo::Unknown);

        self.inputs_unhandled.push(tensor.id);
        arg
    }

    pub fn input(&mut self, tensor: &TensorDescription) -> Arg {
        let precision = tensor.dtype.into();

        // Bool tensors are encoded as bool_precision.
        let precision_input = match precision {
            ElemwisePrecision::Bool => self.bool_precision,
            _ => precision,
        };

        match self.locals.get(precision, tensor.id) {
            Some(local) => {
                self.inputs.update(precision_input, tensor);
                // An input can be an output of a previously fused operation.
                // We need to flag the new status for the tensor.
                self.outputs.update(precision_input, tensor);

                local
            }
            None => {
                let new_input = self.inputs.insert(precision_input, tensor.clone());
                let out = self.locals.create(precision, tensor.id);
                let input = Arg::Input(new_input, precision_input, LayoutInfo::Unknown);

                self.reads.insert(
                    tensor.id,
                    ElemwiseOp::Assign(UnaryElemwiseArgs {
                        input,
                        out: out.clone(),
                    }),
                );

                out
            }
        }
    }

    pub fn output(&mut self, tensor: &TensorDescription) -> Arg {
        let precision = tensor.dtype.into();

        // Bool tensors are encoded as bool_precision.
        let precision_output = match precision {
            ElemwisePrecision::Bool => self.bool_precision,
            _ => precision,
        };

        match self.locals.get(precision, tensor.id) {
            Some(local) => local,
            None => {
                let out = self.locals.create(precision, tensor.id);

                self.outputs.insert(precision_output, tensor.clone());

                out
            }
        }
    }

    pub fn input_reshaped(
        &mut self,
        tensor: &TensorDescription,
        output: &TensorDescription,
    ) -> Option<Arg> {
        let precision = tensor.dtype.into();

        // Bool tensors are encoded as bool_precision.
        let precision_input = match precision {
            ElemwisePrecision::Bool => self.bool_precision,
            _ => precision,
        };

        match self.locals.get(precision, tensor.id) {
            // Can't fused an already fused input.
            //
            // TODO: Can fuse one that is in global memory.
            Some(_) => {
                if self.outputs.get(precision_input, tensor.id).is_some() {
                    return None;
                }

                // self.inputs.update(precision_input, tensor);
                None
            }
            None => {
                let new_input = self.inputs.insert(precision_input, tensor.clone());
                let out = self.locals.create(precision, tensor.id);
                let original = Arg::Input(new_input, precision_input, LayoutInfo::Unknown);

                let mut shape = Sequence::new();

                let index = self.shapes.len();
                self.shapes.push(output.clone());
                let rank = output.shape.len();

                println!("output {output:?}");
                for i in 0..output.shape.len() {
                    let id = index * rank + i;
                    println!("id {id:?}");
                    shape.push(Arg::ScalarShape(id as u32));
                }

                let input = Arg::InputReshaped {
                    original: Box::new(original),
                    shape,
                };

                self.reads.insert(
                    tensor.id,
                    ElemwiseOp::Assign(UnaryElemwiseArgs {
                        input,
                        out: out.clone(),
                    }),
                );

                Some(out)
            }
        }
    }

    pub fn scalar<E: Element>(&mut self, _: &E, dtype: DType) -> Arg {
        let precision = dtype.into();

        // Bool scalars are encoded as bool_precision.
        let precision = match precision {
            ElemwisePrecision::Bool => self.bool_precision,
            _ => precision,
        };
        let new_index = self.scalars.get(&precision).copied().unwrap_or(0);

        let num_scalars = new_index + 1;

        self.scalars.insert(precision, num_scalars);
        Arg::Scalar(new_index, precision)
    }

    pub fn build(&self) -> FuseOnWriteTrace {
        let inputs = self.inputs.clone();
        let outputs = self.output_tensors();
        let ops = self.ops.clone();
        let scalars = self.scalars.clone();
        let reads = self.reads.clone();

        let mut writes = BTreeMap::new();

        for (precision, tensor) in outputs.iter() {
            let local = self.locals.get_any_precision(tensor.id).unwrap();
            let out_index = outputs.get_index(precision, tensor.id).unwrap();

            writes.insert(
                tensor.id,
                ElemwiseOp::Assign(UnaryElemwiseArgs {
                    input: local,
                    out: Arg::Output(out_index as u32, precision, LayoutInfo::Unknown),
                }),
            );
        }

        let shapes = self.shapes.clone();

        // Current problem is that I need btreemap instead of sequences.
        FuseOnWriteTrace::new(
            outputs,
            inputs,
            scalars,
            shapes,
            ops,
            reads,
            writes,
            self.inputs_unhandled.clone(),
        )
    }

    fn output_tensors(&self) -> RegisteredTensors {
        let mut result = RegisteredTensors::default();

        let mut local_tensor_ids_input = Vec::new();
        let mut local_tensor_ids_output = Vec::new();

        // Mark a variable to the provided list of tensor ids using the variable list.
        //
        // Only local variables can become outputs.
        let mark = |var: &Arg, list: &mut Vec<(TensorId, ElemwisePrecision)>| {
            if let Arg::Local(index, precision) = var {
                if let Some(tensor_id) = self.locals.find_tensor_id(*precision, *index) {
                    // Input and outputs tensors are using bool_precision for booleans.
                    let precision = match precision {
                        ElemwisePrecision::Bool => self.bool_precision,
                        _ => *precision,
                    };

                    let entry = (tensor_id, precision);
                    if !list.contains(&entry) {
                        list.push(entry);
                    }
                }
            }
        };

        let mark_binary =
            |op: &BinaryElemwiseArgs,
             inputs: &mut Vec<(TensorId, ElemwisePrecision)>,
             outputs: &mut Vec<(TensorId, ElemwisePrecision)>| {
                mark(&op.lhs, inputs);
                mark(&op.rhs, inputs);
                mark(&op.out, outputs);
            };
        let mark_unary =
            |op: &UnaryElemwiseArgs,
             inputs: &mut Vec<(TensorId, ElemwisePrecision)>,
             outputs: &mut Vec<(TensorId, ElemwisePrecision)>| {
                mark(&op.input, inputs);
                mark(&op.out, outputs);
            };

        let mut mark_op = |op: &ElemwiseOp| match op {
            ElemwiseOp::Add(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Sub(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Mul(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Div(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Powf(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Abs(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Exp(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Log(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Log1p(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Cos(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Sin(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Tanh(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Erf(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Recip(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Assign(op) => mark_unary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::ConditionalAssign {
                cond,
                lhs,
                rhs,
                out,
            } => {
                mark(cond, &mut local_tensor_ids_input);
                mark(lhs, &mut local_tensor_ids_input);
                mark(rhs, &mut local_tensor_ids_input);
                mark(out, &mut local_tensor_ids_output);
            }
            ElemwiseOp::Equal(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Lower(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::Greater(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::LowerEqual(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
            ElemwiseOp::GreaterEqual(op) => mark_binary(
                op,
                &mut local_tensor_ids_input,
                &mut local_tensor_ids_output,
            ),
        };

        // For all operators, mark their local tensor id in the proper set.
        for (_, op) in self.reads.iter() {
            mark_op(op);
        }

        for op in self.ops.iter() {
            mark_op(op);
        }

        for arg in self.outputs_unhandled.iter() {
            mark(arg, &mut local_tensor_ids_output);
        }

        // All output tensors that are never read by a following operation should be written to
        // since they are essentially the "logical" output of the shader.
        for entry in local_tensor_ids_output {
            let is_read = local_tensor_ids_input.contains(&entry);

            if !is_read {
                let (tensor_id, precision) = entry;
                let tensor = self.outputs.get(precision, tensor_id).unwrap();
                result.insert(precision, tensor.clone());
            }
        }

        // All tensors where their latest description is read only should be written to since they
        // are going to be used after the fused kernel by other operations.
        for (precision, tensor) in self.outputs.iter() {
            if let TensorStatus::ReadOnly = tensor.status {
                result.insert(precision, tensor.clone());
            }
        }

        result
    }
}

#[derive(Default, Clone)]
struct Locals {
    values: BTreeMap<ElemwisePrecision, BTreeMap<TensorId, u32>>,
}

impl Locals {
    fn get(&self, precision: ElemwisePrecision, tensor_id: TensorId) -> Option<Arg> {
        if let Some(indexes) = self.values.get(&precision) {
            if let Some(index) = indexes.get(&tensor_id) {
                return Some(Arg::Local(*index, precision));
            }
        }

        None
    }

    fn get_any_precision(&self, tensor_id: TensorId) -> Option<Arg> {
        for (precision, indexes) in self.values.iter() {
            if let Some(index) = indexes.get(&tensor_id) {
                return Some(Arg::Local(*index, *precision));
            }
        }

        None
    }

    fn find_tensor_id(&self, precision: ElemwisePrecision, position: u32) -> Option<TensorId> {
        if let Some(indexes) = self.values.get(&precision) {
            indexes
                .iter()
                .find(|(_id, index)| **index == position)
                .map(|(id, _index)| *id)
        } else {
            None
        }
    }

    fn create(&mut self, precision: ElemwisePrecision, tensor_id: TensorId) -> Arg {
        if let Some(indexes) = self.values.get_mut(&precision) {
            let new_index = indexes.len() as u32;
            indexes.insert(tensor_id, new_index);
            return Arg::Local(new_index, precision);
        }

        let new_index = 0;
        self.values
            .insert(precision, BTreeMap::from_iter([(tensor_id, new_index)]));

        Arg::Local(new_index, precision)
    }
}
