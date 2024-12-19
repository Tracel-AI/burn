use crate::tensor::JitTensor;
use crate::FloatElement;
use crate::{IntElement, JitElement, JitRuntime};
use burn_tensor::quantization::{QuantizationScheme, QuantizationType};
use cubecl::calculate_cube_count_elemwise;
use cubecl::prelude::*;

use super::{
    pack_i8s_into_u32, AFFINE_RANGE_MAX_I8, AFFINE_RANGE_MIN_I8, SYMMETRIC_RANGE_MAX_I8,
    SYMMETRIC_RANGE_MIN_I8,
};

#[cube]
/// Apply int8 affine quantization to the floating-point value.
pub(crate) fn quantize_affine_int8<F: Float>(
    value: Line<F>,
    scale: f32,
    offset: i32,
    range_min: f32,
    range_max: f32,
) -> Line<u32> {
    // x_q = clamp(round(x / scale + offset), a, b)
    // NOTE: we add 256 before casting to unsigned to correctly represent negative values
    Line::cast_from(
        Line::clamp(
            Line::round((value / Line::cast_from(scale)) + Line::cast_from(offset)),
            Line::cast_from(range_min),
            Line::cast_from(range_max),
        ) + Line::cast_from(comptime!(256f32)),
    )
}

#[cube]
/// Apply int8 symmetric quantization to the floating-point value.
pub(crate) fn quantize_symmetric_int8<F: Float>(
    value: Line<F>,
    scale: f32,
    range_min: f32,
    range_max: f32,
) -> Line<u32> {
    // x_q = clamp(round(x / scale), a, b)
    // NOTE: we add 256 before casting to unsigned to correctly represent negative values
    Line::cast_from(
        Line::clamp(
            Line::round(value / Line::cast_from(scale)),
            Line::cast_from(range_min),
            Line::cast_from(range_max),
        ) + Line::cast_from(comptime!(256f32)),
    )
}

#[cube]
/// Apply int8 affine quantization to a line of 4 floating-point values and pack the values into a single u32.
pub(crate) fn quantize_affine_int8_packed<F: Float>(
    value: Line<F>,
    scale: f32,
    offset: i32,
    range_min: f32,
    range_max: f32,
) -> u32 {
    pack_i8s_into_u32(quantize_affine_int8(
        value, scale, offset, range_min, range_max,
    ))
}

#[cube]
/// Apply int8 symmetric quantization to a line of 4 floating-point values and pack the values into a single u32.
pub(crate) fn quantize_symmetric_int8_packed<F: Float>(
    value: Line<F>,
    scale: f32,
    range_min: f32,
    range_max: f32,
) -> u32 {
    pack_i8s_into_u32(quantize_symmetric_int8(value, scale, range_min, range_max))
}

#[cube]
/// Apply int8 affine quantization to a line of floating-point values.
///
/// Each group of 4 resulting quantized values is packed into a single u32 to produce
/// a line of packed quantized values.
pub(crate) fn quantize_affine_int8_packed_line<F: Float>(
    value: Line<F>,
    scale: f32,
    offset: i32,
    range_min: f32,
    range_max: f32,
) -> Line<u32> {
    let line_size = value.size();
    let num_packed = crate::kernel::quantization::NUM_PACKED_QINT8;
    let num_values = line_size / num_packed;
    let mut values = Line::<u32>::empty(num_values);

    #[unroll]
    for i in 0..num_values {
        let mut input = Line::<F>::empty(num_packed);

        #[unroll]
        for j in 0..num_packed {
            input[j] = value[i * num_packed + j];
        }
        values[i] = quantize_affine_int8_packed(input, scale, offset, range_min, range_max);
    }
    values
}

#[cube]
/// Apply int8 symmetric quantization to a line of floating-point values.
///
/// Each group of 4 resulting quantized values is packed into a single u32 to produce
/// a line of packed quantized values.
pub(crate) fn quantize_symmetric_int8_packed_line<F: Float>(
    value: Line<F>,
    scale: f32,
    range_min: f32,
    range_max: f32,
) -> Line<u32> {
    let line_size = value.size();
    let num_packed = crate::kernel::quantization::NUM_PACKED_QINT8;
    let num_values = line_size / num_packed;
    let mut values = Line::<u32>::empty(num_values);

    #[unroll]
    for i in 0..num_values {
        let mut input = Line::<F>::empty(num_packed);

        #[unroll]
        for j in 0..num_packed {
            input[j] = value[i * num_packed + j];
        }
        values[i] = quantize_symmetric_int8_packed(input, scale, range_min, range_max);
    }
    values
}

#[cube(launch_unchecked)]
pub(crate) fn quantize_per_tensor_affine_int8_kernel(
    input: &Tensor<Line<f32>>,
    scale: &Tensor<f32>,
    offset: &Tensor<i32>,
    range_min: f32,
    range_max: f32,
    output: &mut Array<u32>,
) {
    if ABSOLUTE_POS >= output.len() {
        return;
    }

    let scale = scale[0];
    let offset = offset[0];

    // Cast the scale to u32 and write the value in the output
    if ABSOLUTE_POS == output.len() - 1 {
        output[ABSOLUTE_POS] = u32::bitcast_from(scale);
        return;
    }

    // Cast the offset to u32 and write the value in the output
    if ABSOLUTE_POS == output.len() - 2 {
        output[ABSOLUTE_POS] = u32::bitcast_from(offset);
        return;
    }

    let line_size = comptime!(input.line_size());
    if comptime!(line_size == 4) {
        // Assuming a line size of 4 (equal to the number of values packed)
        output[ABSOLUTE_POS] =
            quantize_affine_int8_packed(input[ABSOLUTE_POS], scale, offset, range_min, range_max);
    } else {
        let mut v_packed = 0;
        let num_packed = comptime!(4);
        #[unroll]
        for i in 0..num_packed {
            let v = quantize_affine_int8::<f32>(
                input[ABSOLUTE_POS + i],
                scale,
                offset,
                range_min,
                range_max,
            );
            // Shift and combine into u32
            v_packed |= (v[0] & 0xFF) << (8 * i);
        }
        output[ABSOLUTE_POS] = v_packed;
    }
}

// Would have wrapped symmetric with the same affine kernel but cube doesn't support Option<Tensor> for offset.
#[cube(launch_unchecked)]
pub(crate) fn quantize_per_tensor_symmetric_int8_kernel(
    input: &Tensor<Line<f32>>,
    scale: &Tensor<f32>,
    range_min: f32,
    range_max: f32,
    output: &mut Array<u32>,
) {
    if ABSOLUTE_POS >= output.len() {
        return;
    }

    let scale = scale[0];

    // Cast the scale to u32 and write the value in the output
    if ABSOLUTE_POS == output.len() - 1 {
        output[ABSOLUTE_POS] = u32::bitcast_from(scale);
        return;
    }

    let line_size = comptime!(input.line_size());
    if comptime!(line_size == 4) {
        // Assuming a vectorization factor of 4 (equal to the number of values packed)
        output[ABSOLUTE_POS] =
            quantize_symmetric_int8_packed(input[ABSOLUTE_POS], scale, range_min, range_max);
    } else {
        // Line size of 1
        let mut values = Line::<f32>::empty(crate::kernel::quantization::NUM_PACKED_QINT8);

        #[unroll]
        for i in 0..comptime!(crate::kernel::quantization::NUM_PACKED_QINT8) {
            values[i] = input[ABSOLUTE_POS + i][0];
        }

        output[ABSOLUTE_POS] = quantize_symmetric_int8_packed(values, scale, range_min, range_max);
    }
}

pub(crate) fn quantize_per_tensor<R, F, I>(
    tensor: JitTensor<R>,
    scale: JitTensor<R>,
    offset: Option<JitTensor<R>>,
    scheme: QuantizationScheme,
) -> JitTensor<R>
where
    R: JitRuntime,
    F: JitElement,
    I: IntElement,
{
    let ndims = tensor.shape.num_dims();
    let num_elems = tensor.shape.num_elements();
    let client = tensor.client.clone();
    // Output tensor contains 4x less elements (four int8 values packed in a single u32)
    let output_num_elems = usize::div_ceil(num_elems, 4) * core::mem::size_of::<u32>();

    // Force vectorization to process 4 quantized values packed for 1 output value
    let line_size: u8 = if num_elems < 4 { 1 } else { 4 };
    let cube_dim = CubeDim::default();
    let cube_count = calculate_cube_count_elemwise(num_elems / line_size as usize, cube_dim);

    let dummy_array = vec![1; ndims];
    if let Some(offset) = offset {
        // Scale and offset qparams are also packed in the tensor dat
        let handle = client
            .empty(output_num_elems + core::mem::size_of::<f32>() + core::mem::size_of::<i32>());
        let output = JitTensor::new_contiguous(
            client.clone(),
            tensor.device.clone(),
            tensor.shape.clone(),
            handle,
            burn_tensor::DType::QFloat(scheme),
        );

        unsafe {
            quantize_per_tensor_affine_int8_kernel::launch_unchecked::<R>(
                &client,
                cube_count,
                cube_dim,
                tensor.as_tensor_arg::<F>(line_size),
                // Ignore shape and stride
                TensorArg::from_raw_parts::<F>(&scale.handle, &dummy_array, &dummy_array, 1),
                TensorArg::from_raw_parts::<I>(&offset.handle, &dummy_array, &dummy_array, 1),
                ScalarArg::new(AFFINE_RANGE_MIN_I8),
                ScalarArg::new(AFFINE_RANGE_MAX_I8),
                output.as_array_arg::<u32>(1),
            )
        };
        output
    } else {
        // Scale qparam is also packed in the tensor data
        let handle = client.empty(output_num_elems + core::mem::size_of::<f32>());
        let output = JitTensor::new_contiguous(
            client.clone(),
            tensor.device.clone(),
            tensor.shape.clone(),
            handle,
            burn_tensor::DType::QFloat(scheme),
        );

        unsafe {
            quantize_per_tensor_symmetric_int8_kernel::launch_unchecked::<R>(
                &client,
                cube_count,
                cube_dim,
                tensor.as_tensor_arg::<F>(line_size),
                // Ignore shape and stride
                TensorArg::from_raw_parts::<F>(&scale.handle, &dummy_array, &dummy_array, 1),
                ScalarArg::new(SYMMETRIC_RANGE_MIN_I8),
                ScalarArg::new(SYMMETRIC_RANGE_MAX_I8),
                output.as_array_arg::<u32>(1),
            )
        };

        output
    }
}

/// Convert the tensor to a lower precision data type based on the quantization scheme and parameters.
pub fn quantize<R, F, I>(
    tensor: JitTensor<R>,
    scheme: &QuantizationScheme,
    scale: JitTensor<R>,
    offset: Option<JitTensor<R>>,
) -> JitTensor<R>
where
    R: JitRuntime,
    F: FloatElement,
    I: IntElement,
{
    match scheme {
        QuantizationScheme::PerTensorAffine(dtype)
        | QuantizationScheme::PerTensorSymmetric(dtype) => match dtype {
            QuantizationType::QInt8 => {
                quantize_per_tensor::<R, F, I>(tensor, scale, offset, *scheme)
            }
        },
    }
}
