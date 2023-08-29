use burn_tensor::ops::{
    ConvOptions, ConvTransposeOptions, MaxPool2dBackward, MaxPool2dWithIndices, ModuleOps,
};

use crate::{
    element::{CandleElement, FloatCandleElement, IntCandleElement},
    CandleBackend, CandleTensor,
};

use super::base::{FloatTensor, IntTensor};

impl<F: FloatCandleElement, I: IntCandleElement> ModuleOps<CandleBackend<F, I>>
    for CandleBackend<F, I>
{
    fn conv1d(
        x: FloatTensor<Self, 3>,
        weight: FloatTensor<Self, 3>,
        bias: Option<FloatTensor<Self, 1>>,
        options: ConvOptions<1>,
    ) -> FloatTensor<Self, 3> {
        assert!(
            options.dilation[0] == 1,
            "Candle does not support dilation different from 1 in convolutions"
        );
        let conv = x
            .tensor
            .conv1d(
                &weight.tensor,
                options.padding[0],
                options.stride[0],
                options.groups,
            )
            .unwrap();
        CandleTensor::new(match bias {
            Some(bias) => conv
                .broadcast_add(&bias.tensor.unsqueeze(1).unwrap())
                .unwrap(),
            None => conv,
        })
    }

    fn conv2d(
        x: FloatTensor<Self, 4>,
        weight: FloatTensor<Self, 4>,
        bias: Option<FloatTensor<Self, 1>>,
        options: ConvOptions<2>,
    ) -> FloatTensor<Self, 4> {
        assert!(
            options.dilation[0] == 1 && options.dilation[1] == 1,
            "Candle does not support dilation in convolutions"
        );
        assert!(
            options.padding[0] == options.padding[1],
            "Candle does not support per dimension padding in convolutions"
        );
        assert!(
            options.stride[0] == options.stride[1],
            "Candle does not support per dimension stride in convolutions"
        );
        let conv = x
            .tensor
            .conv2d(
                &weight.tensor,
                options.padding[0],
                options.stride[0],
                options.groups,
            )
            .unwrap();
        CandleTensor::new(match bias {
            Some(bias) => conv
                .broadcast_add(
                    &bias
                        .tensor
                        .unsqueeze(0)
                        .unwrap()
                        .unsqueeze(2)
                        .unwrap()
                        .unsqueeze(3)
                        .unwrap(),
                )
                .unwrap(),
            None => conv,
        })
    }

    fn conv_transpose2d(
        x: FloatTensor<Self, 4>,
        weight: FloatTensor<Self, 4>,
        bias: Option<FloatTensor<Self, 1>>,
        options: ConvTransposeOptions<2>,
    ) -> FloatTensor<Self, 4> {
        panic!("conv_transpose is not supported by Candle")
    }

    fn avg_pool2d(
        x: FloatTensor<Self, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        count_include_pad: bool,
    ) -> FloatTensor<Self, 4> {
        assert!(
            padding[0] == 0 && padding[1] == 0,
            "Candle does not support padding in pooling"
        );
        assert!(
            count_include_pad,
            "Candle does not support excluding pad count in pooling"
        );
        CandleTensor::new(
            x.tensor
                .avg_pool2d(kernel_size.into(), stride.into())
                .unwrap(),
        )
    }

    fn avg_pool2d_backward(
        x: FloatTensor<Self, 4>,
        grad: FloatTensor<Self, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        count_include_pad: bool,
    ) -> FloatTensor<Self, 4> {
        panic!("avg_pool2d_backward is not supported by Candle")
    }

    fn max_pool2d(
        x: FloatTensor<Self, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        dilation: [usize; 2],
    ) -> FloatTensor<Self, 4> {
        assert!(
            padding[0] == 0 && padding[1] == 0,
            "Candle does not support padding in pooling"
        );
        assert!(
            dilation[0] == 1 && dilation[1] == 1,
            "Candle does not support dilation in pooling"
        );
        CandleTensor::new(
            x.tensor
                .max_pool2d(kernel_size.into(), stride.into())
                .unwrap(),
        )
    }

    fn max_pool2d_with_indices(
        x: FloatTensor<Self, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        dilation: [usize; 2],
    ) -> MaxPool2dWithIndices<CandleBackend<F, I>> {
        panic!("max_pool2d_with_indices is not supported by Candle")
    }

    fn max_pool2d_with_indices_backward(
        x: FloatTensor<Self, 4>,
        kernel_size: [usize; 2],
        stride: [usize; 2],
        padding: [usize; 2],
        dilation: [usize; 2],
        output_grad: FloatTensor<Self, 4>,
        indices: IntTensor<Self, 4>,
    ) -> MaxPool2dBackward<CandleBackend<F, I>> {
        panic!("max_pool2d_with_indices_backward is not supported by Candle")
    }

    fn adaptive_avg_pool2d(
        x: FloatTensor<Self, 4>,
        output_size: [usize; 2],
    ) -> FloatTensor<Self, 4> {
        panic!("adaptive_avg_pool2 is not supported by Candle")
    }

    fn adaptive_avg_pool2d_backward(
        x: FloatTensor<Self, 4>,
        grad: FloatTensor<Self, 4>,
    ) -> FloatTensor<Self, 4> {
        panic!("adaptive_avg_pool2d_backward is not supported by Candle")
    }
}
