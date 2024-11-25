use crate::{element::BasicJitElement, ops::empty_device, tensor::JitTensor, JitRuntime};
use cubecl::{calculate_cube_count_elemwise, prelude::*};

#[cube(launch_unchecked)]
fn flip_kernel<E: CubePrimitive, Bool: Int>(
    input: &Tensor<E>,
    output: &mut Tensor<E>,
    indices: Sequence<Bool>,
    #[comptime] rank: u32,
) {
    if ABSOLUTE_POS >= output.len() {
        return;
    }

    let mut offset_input = 0;

    #[unroll]
    for i in 0..rank {
        let stride = input.stride(i);
        let shape = output.shape(i);
        let flip = *indices.index(i) == Bool::from_int(1);
        let mut offset_local = ABSOLUTE_POS / stride % shape;

        if flip {
            offset_local = shape - offset_local - 1;
        }

        offset_input += offset_local * stride;
    }

    output[ABSOLUTE_POS] = input[offset_input];
}

pub(crate) fn flip<R: JitRuntime, E: BasicJitElement>(
    tensor: JitTensor<R>,
    indices: &[usize],
) -> JitTensor<R> {
    let output = empty_device::<R, E>(
        tensor.client.clone(),
        tensor.device.clone(),
        tensor.shape.clone(),
    );
    flip_on_output::<R, E>(tensor, output, indices)
}

pub(crate) fn flip_on_output<R: JitRuntime, E: BasicJitElement>(
    tensor: JitTensor<R>,
    output: JitTensor<R>,
    indices: &[usize],
) -> JitTensor<R> {
    let ndims = tensor.shape.num_dims();
    let mut indices_sequence = SequenceArg::<'_, R, u32>::new();

    for i in 0..ndims {
        indices_sequence.push(ScalarArg::new(indices.contains(&i) as u32));
    }

    let cube_dim = CubeDim::default();
    let cube_count = calculate_cube_count_elemwise(output.shape.num_elements(), cube_dim);

    unsafe {
        flip_kernel::launch_unchecked::<E, u32, R>(
            &tensor.client,
            cube_count,
            cube_dim,
            tensor.as_tensor_arg::<E>(1),
            output.as_tensor_arg::<E>(1),
            indices_sequence,
            ndims as u32,
        );
    }

    output
}
