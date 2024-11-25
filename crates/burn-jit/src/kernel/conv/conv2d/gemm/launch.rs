use burn_tensor::{
    ops::{conv::calculate_conv_output_size, ConvOptions},
    Shape,
};
use cubecl::{
    ir::{Elem, FloatKind},
    linalg::matmul::{
        self,
        components::{
            stage::{S2x2x8, S4x2x4, S8x2x4},
            MatrixLayout,
        },
    },
    tensor_line_size,
};

use crate::{
    kernel::{
        conv::{
            conv2d::gemm::{
                algorithm::{Algorithm, Cmma},
                base::{ConvolutionLaunch, ConvolutionProblem},
            },
            nchw_to_nhwc, Conv2dAutotuneKey,
        },
        into_contiguous,
    },
    ops::{
        into_data_sync,
        numeric::{empty_device, zeros_device},
        permute, reshape,
    },
    tensor::JitTensor,
    FloatElement, IntElement, JitRuntime,
};

use super::algorithm::CmmaHalf;

pub type LargeMAlgorithm<F> = Cmma<F, S8x2x4>;
pub type BalancedAlgorithm<F> = Cmma<F, S4x2x4>;

/// Perform a 2D convolution using the GEMM (im2col) algorithm.
///
/// * `input` - The input feature map
/// * `weight` - The weights (filter) applied to each kernel
/// * `bias` - The bias added to each channel
/// * `options` - The options to use for the convolution
///
///
#[allow(clippy::extra_unused_type_parameters)]
pub fn conv2d_gemm_large_m<R: JitRuntime, F: FloatElement, I: IntElement>(
    input: JitTensor<R>,
    weight: JitTensor<R>,
    bias: Option<JitTensor<R>>,
    options: ConvOptions<2>,
) -> JitTensor<R> {
    match F::as_elem() {
        Elem::Float(FloatKind::F16) => {
            conv2d_gemm_with_algo::<R, F, CmmaHalf<F, S8x2x4>>(input, weight, bias, options)
        }
        _ => conv2d_gemm_with_algo::<R, F, LargeMAlgorithm<F>>(input, weight, bias, options),
    }
}

#[allow(clippy::extra_unused_type_parameters)]
pub fn conv2d_gemm_balanced<R: JitRuntime, F: FloatElement, I: IntElement>(
    input: JitTensor<R>,
    weight: JitTensor<R>,
    bias: Option<JitTensor<R>>,
    options: ConvOptions<2>,
) -> JitTensor<R> {
    match F::as_elem() {
        Elem::Float(FloatKind::F16) => {
            conv2d_gemm_with_algo::<R, F, CmmaHalf<F, S4x2x4>>(input, weight, bias, options)
        }
        _ => conv2d_gemm_with_algo::<R, F, BalancedAlgorithm<F>>(input, weight, bias, options),
    }
}

pub fn conv2d_gemm_with_algo<R: JitRuntime, F: FloatElement, Alg: Algorithm<F>>(
    input: JitTensor<R>,
    weight: JitTensor<R>,
    bias: Option<JitTensor<R>>,
    options: ConvOptions<2>,
) -> JitTensor<R> {
    let [batch_size, in_channels, height, width] = input.shape.dims();
    let [out_channels, _, kernel_h, kernel_w] = weight.shape.dims();

    let out_h = calculate_conv_output_size(
        kernel_h,
        options.stride[0],
        options.padding[0],
        options.dilation[0],
        height,
    );
    let out_w = calculate_conv_output_size(
        kernel_w,
        options.stride[1],
        options.padding[1],
        options.dilation[1],
        width,
    );

    let input = match input.is_contiguous() {
        true => nchw_to_nhwc::<R, F>(input),
        false => into_contiguous(permute(input, &[0, 2, 3, 1])),
    };
    let weight = into_contiguous(permute(weight, &[2, 3, 1, 0]));

    // Implicit GEMM matrix size
    let gemm_m = batch_size * out_h * out_w;
    let gemm_n = out_channels;
    let gemm_k = kernel_h * kernel_w * in_channels;

    let weight = reshape(weight, Shape::new([gemm_k, gemm_n]));

    let out_shape = Shape::new([gemm_m, gemm_n]);
    let out = empty_device::<R, F>(input.client.clone(), input.device.clone(), out_shape);

    // Target 128 bit accesses
    let available_vectorizations = R::supported_line_sizes()
        .iter()
        .copied()
        .filter(|it| *it as usize * size_of::<F>() <= 16)
        .collect::<Vec<_>>();
    let lhs_line_size = tensor_line_size(
        &available_vectorizations,
        &input.shape.dims,
        &input.strides,
        3,
    );
    let rhs_line_size = tensor_line_size(
        &available_vectorizations,
        &weight.shape.dims,
        &weight.strides,
        1,
    );
    let out_line_size =
        tensor_line_size(&available_vectorizations, &out.shape.dims, &out.strides, 1);

    let problem = ConvolutionProblem {
        m: gemm_m,
        n: gemm_n,
        k: gemm_k,
        lhs_layout: matmul::components::MatrixLayout::RowMajor,
        rhs_layout: matmul::components::MatrixLayout::RowMajor,
        lhs_line_size,
        rhs_line_size,
        out_line_size,

        kernel_size: (kernel_h as u32, kernel_w as u32),
        options,
        out_shape_y: out_h,
        out_shape_x: out_w,

        has_bias: bias.is_some(),
    };

    if !Alg::can_launch::<R>(&input.client, &problem) {
        panic!("Can't do implicit GEMM");
    }

    let cube_dim = Alg::cube_dim();
    let cube_count = Alg::cube_count(&problem);

    let advanced_config = Default::default();
    let config = Alg::make_config(&problem, &cube_dim, &cube_count, &advanced_config);
    let bias = bias.unwrap_or_else(|| {
        empty_device::<R, F>(input.client.clone(), input.device.clone(), Shape::new([1]))
    });

    println!("input: {}", into_data_sync::<R, F>(input.clone()));
    println!("input shape: {:?}", input.shape.dims);
    println!("weight: {}", into_data_sync::<R, F>(weight.clone()));
    println!("weight shape: {:?}", weight.shape.dims);
    println!("bias: {}", into_data_sync::<R, F>(bias.clone()));
    println!("config: {config:#?}");

    let test_shape = Shape::new([batch_size, out_h, out_w, kernel_h, kernel_w, in_channels]);
    let test = zeros_device::<R, F>(out.client.clone(), out.device.clone(), test_shape);

    unsafe {
        Alg::GlobalMatmul::launch_unchecked::<R>(
            &input.client,
            cube_dim,
            cube_count,
            input.as_tensor_arg::<F>(lhs_line_size),
            weight.as_tensor_arg::<F>(rhs_line_size),
            bias.as_tensor_arg::<F>(out_line_size),
            out.as_tensor_arg::<F>(out_line_size),
            test.as_tensor_arg::<F>(1),
            config,
        );
    }

    println!("out: {}", into_data_sync::<R, F>(out.clone()));
    println!("test: {}", into_data_sync::<R, F>(test));

    // Reset to NCHW
    let out = reshape(out, Shape::new([batch_size, out_h, out_w, out_channels]));
    permute(out, &[0, 3, 1, 2])
}

pub fn problem_from_key<R: JitRuntime, F: FloatElement>(
    key: &Conv2dAutotuneKey,
    out_h: usize,
    out_w: usize,
) -> ConvolutionProblem {
    let in_stride_2 = key.in_channels;
    let in_stride_1 = key.width * in_stride_2;
    let in_stride_0 = key.height * in_stride_1;

    let m = key.batch_size * out_h * out_w;
    let n = key.out_channels;
    let k = key.kernel_size[0] * key.kernel_size[1] * key.in_channels;

    let options = ConvOptions {
        stride: key.stride,
        padding: key.padding,
        dilation: key.dilation,
        groups: key.groups,
    };

    // Target 128 bit accesses
    let available_vectorizations = R::supported_line_sizes()
        .iter()
        .copied()
        .filter(|it| *it as usize * size_of::<F>() <= 16)
        .collect::<Vec<_>>();
    let lhs_line_size = tensor_line_size(
        &available_vectorizations,
        &[key.batch_size, key.height, key.width, key.in_channels],
        &[in_stride_0, in_stride_1, in_stride_2, 1],
        3,
    );
    let rhs_line_size = tensor_line_size(&available_vectorizations, &[k, n], &[n, 1], 1);
    let out_line_size = tensor_line_size(&available_vectorizations, &[m, n], &[n, 1], 1);

    ConvolutionProblem {
        m,
        n,
        k,
        lhs_layout: MatrixLayout::RowMajor,
        rhs_layout: MatrixLayout::RowMajor,
        lhs_line_size,
        rhs_line_size,
        out_line_size,
        kernel_size: (key.kernel_size[0] as u32, key.kernel_size[1] as u32),
        options,
        out_shape_y: out_h,
        out_shape_x: out_w,
        has_bias: key.has_bias,
    }
}
