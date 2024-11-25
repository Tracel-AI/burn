use super::pool2d::{pool2d_direct, Pool2dDirectArgsLaunch, Pool2dDirectStrategy};
use crate::{element::JitElement, ops::empty_device, tensor::JitTensor, JitRuntime};
use burn_tensor::{ops::conv::calculate_pool_output_size, Shape};
use cubecl::prelude::*;
use cubecl::{calculate_cube_count_elemwise, prelude::ScalarArg, CubeDim};

struct AvgPoolStrategy;

#[derive(CubeType, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct AvgPoolStrategyConfig {
    kernel_size_h: u32,
    kernel_size_w: u32,
    count_include_pad: bool,
}

#[cube]
impl<T: Algebraic> Pool2dDirectStrategy<T> for AvgPoolStrategy {
    type Accumulator = (T, u32);
    type Config = AvgPoolStrategyConfig;
    type Indices = ();

    fn initialize(#[comptime] config: Self::Config) -> Self::Accumulator {
        let sum = T::from_int(0);
        let count = comptime! {if config.count_include_pad {
            config.kernel_size_h * config.kernel_size_w
        } else {
            0u32
        }};

        (sum, count)
    }

    fn accumulate(
        #[comptime] config: Self::Config,
        accumulator: &mut Self::Accumulator,
        _index: u32,
        result: T,
    ) {
        let (sum, count) = accumulator;

        if comptime![!config.count_include_pad] {
            *count += 1;
        }

        *sum += result;
    }

    fn store(
        #[comptime] _config: Self::Config,
        position: u32,
        output: &mut Tensor<T>,
        _output_indices: &mut (),
        accumulator: Self::Accumulator,
    ) {
        let (sum, count) = accumulator;
        output[position] = sum / T::cast_from(count);
    }
}

pub(crate) fn avg_pool2d<R: JitRuntime, E: JitElement>(
    x: JitTensor<R>,
    kernel_size: [usize; 2],
    stride: [usize; 2],
    padding: [usize; 2],
    count_include_pad: bool,
) -> JitTensor<R> {
    let [batch_size, channels, _, _] = x.shape.dims();
    let dilation = 1;

    let size_0 = calculate_pool_output_size(
        kernel_size[0],
        stride[0],
        padding[0],
        dilation,
        x.shape.dims[2],
    );
    let size_1 = calculate_pool_output_size(
        kernel_size[1],
        stride[1],
        padding[1],
        dilation,
        x.shape.dims[3],
    );

    let shape_out = Shape::new([batch_size, channels, size_0, size_1]);
    let output = empty_device::<R, E>(x.client.clone(), x.device.clone(), shape_out);

    let cube_dim = CubeDim::default();
    let cube_count = calculate_cube_count_elemwise(output.shape.num_elements(), cube_dim);

    pool2d_direct::launch::<E, AvgPoolStrategy, R>(
        &x.client,
        cube_count,
        cube_dim,
        x.as_tensor_arg::<E>(1),
        output.as_tensor_arg::<E>(1),
        (),
        Pool2dDirectArgsLaunch::new(
            ScalarArg::new(stride[0] as u32),
            ScalarArg::new(stride[1] as u32),
            ScalarArg::new(dilation as u32),
            ScalarArg::new(dilation as u32),
            ScalarArg::new(padding[0] as u32),
            ScalarArg::new(padding[1] as u32),
        ),
        (kernel_size[0] as u32, kernel_size[1] as u32),
        AvgPoolStrategyConfig {
            kernel_size_h: kernel_size[0] as u32,
            kernel_size_w: kernel_size[1] as u32,
            count_include_pad,
        },
    );

    output
}
