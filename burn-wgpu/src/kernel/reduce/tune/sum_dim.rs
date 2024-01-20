use burn_compute::tune::{AutotuneOperation, AutotuneOperationSet};
use burn_tensor::{Element, ElementConversion};

use crate::{
    compute::WgpuAutotuneKey,
    element::WgpuElement,
    kernel::{
        prng::{random_like_uniform, random_like_uniform_int},
        reduce::{init_reduce_output, int_sum_dim, sum_dim, sum_dim_shared_memory},
    },
    ops::numeric::empty_device,
    reduce_tune_int_ops, reduce_tune_ops,
    tensor::WgpuTensor,
    IntElement,
};

use super::ReduceAutotuneKey;

/// Set of sum_dim implementations available for autotune
/// Autotune key is given by concatenating the closest upper power of 2 of
/// dim to reduce, and product of others
pub struct SumDimAutotuneOperationSet<E: WgpuElement, const D: usize> {
    key: WgpuAutotuneKey,
    input: WgpuTensor<E, D>,
    output: WgpuTensor<E, D>,
    reduce_dim: usize,
}
impl<E: WgpuElement, const D: usize> SumDimAutotuneOperationSet<E, D> {
    fn new(input: WgpuTensor<E, D>, output: WgpuTensor<E, D>, reduce_dim: usize) -> Self {
        Self {
            key: WgpuAutotuneKey::SumDim(ReduceAutotuneKey::new(
                &input.shape,
                &input.strides,
                reduce_dim,
            )),
            input,
            output,
            reduce_dim,
        }
    }
}

impl<E: WgpuElement + Element, const D: usize> AutotuneOperationSet<WgpuAutotuneKey>
    for SumDimAutotuneOperationSet<E, D>
{
    fn key(&self) -> WgpuAutotuneKey {
        self.key.clone()
    }

    fn autotunables(&self) -> Vec<Box<dyn AutotuneOperation>> {
        let random_bounds: (E, E) = ((-10.0).elem::<E>(), (10.0).elem::<E>());
        let input = random_like_uniform(&self.input, random_bounds.0, random_bounds.1);

        let output = empty_device(
            self.output.client.clone(),
            self.output.device.clone(),
            self.output.shape.clone(),
        );

        vec![
            Box::new(SumDimAutotune::<E, D>::new(
                input.clone(),
                output.clone(),
                self.reduce_dim,
            )),
            Box::new(SumDimSharedMemoryAutotune::<E, D>::new(
                input.clone(),
                output.clone(),
                self.reduce_dim,
            )),
        ]
    }

    fn fastest(self: Box<Self>, fastest_index: usize) -> Box<dyn AutotuneOperation> {
        // Warning: since AutotuneOperationSet shares his key with MeanDimAutotuneOperationSet
        // we must make sure the order here is correlated with MeanDim
        match fastest_index {
            0 => Box::new(SumDimAutotune::<E, D>::new(
                self.input,
                self.output,
                self.reduce_dim,
            )),
            1 => Box::new(SumDimSharedMemoryAutotune::<E, D>::new(
                self.input,
                self.output,
                self.reduce_dim,
            )),
            _ => panic!("Fastest index is out of bound"),
        }
    }
}

/// Executes autotune on sum_dim operation
pub fn sum_dim_autotune<E: WgpuElement + Element, const D: usize>(
    input: WgpuTensor<E, D>,
    reduce_dim: usize,
) -> WgpuTensor<E, D> {
    let client = input.client.clone();

    let output = init_reduce_output(&input, reduce_dim);

    let operation_set = Box::new(SumDimAutotuneOperationSet::<E, D>::new(
        input,
        output.clone(),
        reduce_dim,
    ));

    client.execute_autotune(operation_set);

    output
}

/// Set of sum_dim implementations available for autotuning int tensors
pub struct SumDimIntAutotuneOperationSet<I: IntElement, const D: usize> {
    key: WgpuAutotuneKey,
    input: WgpuTensor<I, D>,
    output: WgpuTensor<I, D>,
    reduce_dim: usize,
}

impl<I: IntElement, const D: usize> SumDimIntAutotuneOperationSet<I, D> {
    fn new(input: WgpuTensor<I, D>, output: WgpuTensor<I, D>, reduce_dim: usize) -> Self {
        Self {
            key: WgpuAutotuneKey::SumDim(ReduceAutotuneKey::new(
                &input.shape,
                &input.strides,
                reduce_dim,
            )),
            input,
            output,
            reduce_dim,
        }
    }
}

impl<I: IntElement, const D: usize> AutotuneOperationSet<WgpuAutotuneKey>
    for SumDimIntAutotuneOperationSet<I, D>
{
    fn key(&self) -> WgpuAutotuneKey {
        self.key.clone()
    }

    fn autotunables(&self) -> Vec<Box<dyn AutotuneOperation>> {
        let random_bounds: (I, I) = ((-10).elem::<I>(), (10).elem::<I>());
        let input: WgpuTensor<I, D> =
            random_like_uniform_int(&self.input, random_bounds.0, random_bounds.1);

        let output = empty_device(
            self.output.client.clone(),
            self.output.device.clone(),
            self.output.shape.clone(),
        );

        vec![
            Box::new(SumDimIntAutotune::<I, D>::new(
                input.clone(),
                output.clone(),
                self.reduce_dim,
            )),
            Box::new(SumDimIntSharedMemoryAutotune::<I, D>::new(
                input.clone(),
                output.clone(),
                self.reduce_dim,
            )),
        ]
    }

    fn fastest(self: Box<Self>, fastest_index: usize) -> Box<dyn AutotuneOperation> {
        // Warning: since AutotuneOperationSet shares his key with MeanDimAutotuneOperationSet
        // we must make sure the order here is correlated with MeanDim
        match fastest_index {
            0 => Box::new(SumDimIntAutotune::<I, D>::new(
                self.input,
                self.output,
                self.reduce_dim,
            )),
            1 => Box::new(SumDimIntSharedMemoryAutotune::<I, D>::new(
                self.input,
                self.output,
                self.reduce_dim,
            )),
            _ => panic!("Fastest index is out of bound"),
        }
    }
}

/// Executes autotune on sum_dim operation
pub fn int_sum_dim_autotune<I: IntElement, const D: usize>(
    input: WgpuTensor<I, D>,
    reduce_dim: usize,
) -> WgpuTensor<I, D> {
    let client = input.client.clone();

    let output = init_reduce_output(&input, reduce_dim);

    let operation_set = Box::new(SumDimIntAutotuneOperationSet::<I, D>::new(
        input,
        output.clone(),
        reduce_dim,
    ));

    client.execute_autotune(operation_set);

    output
}

// Probably better on balanced tensor shapes
reduce_tune_ops!(SumDimAutotune, sum_dim);
reduce_tune_int_ops!(SumDimIntAutotune, int_sum_dim);

// Probably better on tensors large along reduce dim
reduce_tune_ops!(SumDimSharedMemoryAutotune, sum_dim_shared_memory);
reduce_tune_int_ops!(SumDimIntSharedMemoryAutotune, sum_dim_shared_memory);
