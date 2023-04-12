#[burn_tensor_testgen::testgen(ad_conv2d)]
mod tests {
    use super::*;
    use burn_tensor::{module::conv2d, Data};

    #[test]
    fn test_conv2d_basic() {
        let test = Conv2dTestCase {
            batch_size: 2,
            channels_in: 3,
            channels_out: 3,
            kernel_size_1: 3,
            kernel_size_2: 3,
            padding_1: 1,
            padding_2: 1,
            stride_1: 1,
            stride_2: 1,
            height: 6,
            width: 6,
        };
        let grads = Grads {
            x: TestTensor::from_floats([
                [
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                ],
                [
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                ],
            ]),
            weight: TestTensor::from_floats([
                [
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                ],
                [
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                ],
                [
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                ],
            ]),
            bias: TestTensor::from_floats([72., 72., 72.]),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv2d_different_channels() {
        let test = Conv2dTestCase {
            batch_size: 2,
            channels_in: 2,
            channels_out: 3,
            kernel_size_1: 3,
            kernel_size_2: 3,
            padding_1: 1,
            padding_2: 1,
            stride_1: 1,
            stride_2: 1,
            height: 6,
            width: 6,
        };
        let grads = Grads {
            x: TestTensor::from_floats([
                [
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                ],
                [
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                    [
                        [12., 18., 18., 18., 18., 12.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [18., 27., 27., 27., 27., 18.],
                        [12., 18., 18., 18., 18., 12.],
                    ],
                ],
            ]),
            weight: TestTensor::from_floats([
                [
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                ],
                [
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                ],
                [
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                    [[50., 60., 50.], [60., 72., 60.], [50., 60., 50.]],
                ],
            ]),
            bias: TestTensor::from_floats([72., 72., 72.]),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv2d_different_kernel_size() {
        let test = Conv2dTestCase {
            batch_size: 2,
            channels_in: 2,
            channels_out: 2,
            kernel_size_1: 3,
            kernel_size_2: 4,
            padding_1: 1,
            padding_2: 1,
            stride_1: 1,
            stride_2: 1,
            height: 6,
            width: 6,
        };
        let grads = Grads {
            x: TestTensor::from_floats([
                [
                    [
                        [8., 12., 16., 16., 12., 8.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [8., 12., 16., 16., 12., 8.],
                    ],
                    [
                        [8., 12., 16., 16., 12., 8.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [8., 12., 16., 16., 12., 8.],
                    ],
                ],
                [
                    [
                        [8., 12., 16., 16., 12., 8.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [8., 12., 16., 16., 12., 8.],
                    ],
                    [
                        [8., 12., 16., 16., 12., 8.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [12., 18., 24., 24., 18., 12.],
                        [8., 12., 16., 16., 12., 8.],
                    ],
                ],
            ]),
            weight: TestTensor::from_floats([
                [
                    [
                        [40., 50., 50., 40.],
                        [48., 60., 60., 48.],
                        [40., 50., 50., 40.],
                    ],
                    [
                        [40., 50., 50., 40.],
                        [48., 60., 60., 48.],
                        [40., 50., 50., 40.],
                    ],
                ],
                [
                    [
                        [40., 50., 50., 40.],
                        [48., 60., 60., 48.],
                        [40., 50., 50., 40.],
                    ],
                    [
                        [40., 50., 50., 40.],
                        [48., 60., 60., 48.],
                        [40., 50., 50., 40.],
                    ],
                ],
            ]),
            bias: TestTensor::from_floats([60., 60.]),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv2d_different_padding() {
        let test = Conv2dTestCase {
            batch_size: 2,
            channels_in: 2,
            channels_out: 2,
            kernel_size_1: 3,
            kernel_size_2: 3,
            padding_1: 1,
            padding_2: 2,
            stride_1: 1,
            stride_2: 1,
            height: 6,
            width: 6,
        };
        let grads = Grads {
            x: TestTensor::from_floats([
                [
                    [
                        [12., 12., 12., 12., 12., 12.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [12., 12., 12., 12., 12., 12.],
                    ],
                    [
                        [12., 12., 12., 12., 12., 12.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [12., 12., 12., 12., 12., 12.],
                    ],
                ],
                [
                    [
                        [12., 12., 12., 12., 12., 12.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [12., 12., 12., 12., 12., 12.],
                    ],
                    [
                        [12., 12., 12., 12., 12., 12.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [18., 18., 18., 18., 18., 18.],
                        [12., 12., 12., 12., 12., 12.],
                    ],
                ],
            ]),
            weight: TestTensor::from_floats([
                [
                    [[60., 60., 60.], [72., 72., 72.], [60., 60., 60.]],
                    [[60., 60., 60.], [72., 72., 72.], [60., 60., 60.]],
                ],
                [
                    [[60., 60., 60.], [72., 72., 72.], [60., 60., 60.]],
                    [[60., 60., 60.], [72., 72., 72.], [60., 60., 60.]],
                ],
            ]),
            bias: TestTensor::from_floats([96., 96.]),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv2d_different_width() {
        let test = Conv2dTestCase {
            batch_size: 2,
            channels_in: 2,
            channels_out: 2,
            kernel_size_1: 3,
            kernel_size_2: 3,
            padding_1: 1,
            padding_2: 1,
            stride_1: 1,
            stride_2: 1,
            height: 6,
            width: 5,
        };
        let grads = Grads {
            x: TestTensor::from_floats([
                [
                    [
                        [8., 12., 12., 12., 8.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [8., 12., 12., 12., 8.],
                    ],
                    [
                        [8., 12., 12., 12., 8.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [8., 12., 12., 12., 8.],
                    ],
                ],
                [
                    [
                        [8., 12., 12., 12., 8.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [8., 12., 12., 12., 8.],
                    ],
                    [
                        [8., 12., 12., 12., 8.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [12., 18., 18., 18., 12.],
                        [8., 12., 12., 12., 8.],
                    ],
                ],
            ]),
            weight: TestTensor::from_floats([
                [
                    [[40., 50., 40.], [48., 60., 48.], [40., 50., 40.]],
                    [[40., 50., 40.], [48., 60., 48.], [40., 50., 40.]],
                ],
                [
                    [[40., 50., 40.], [48., 60., 48.], [40., 50., 40.]],
                    [[40., 50., 40.], [48., 60., 48.], [40., 50., 40.]],
                ],
            ]),
            bias: TestTensor::from_floats([60., 60.]),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv2d_stride_2() {
        let test = Conv2dTestCase {
            batch_size: 2,
            channels_in: 2,
            channels_out: 2,
            kernel_size_1: 3,
            kernel_size_2: 3,
            padding_1: 1,
            padding_2: 1,
            stride_1: 2,
            stride_2: 2,
            height: 8,
            width: 8,
        };
        let grads = Grads {
            x: TestTensor::from_floats([
                [
                    [
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                    ],
                    [
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                    ],
                ],
                [
                    [
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                    ],
                    [
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [4., 8., 4., 8., 4., 8., 4., 4.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                        [2., 4., 2., 4., 2., 4., 2., 2.],
                    ],
                ],
            ]),
            weight: TestTensor::from_floats([
                [
                    [[18., 24., 24.], [24., 32., 32.], [24., 32., 32.]],
                    [[18., 24., 24.], [24., 32., 32.], [24., 32., 32.]],
                ],
                [
                    [[18., 24., 24.], [24., 32., 32.], [24., 32., 32.]],
                    [[18., 24., 24.], [24., 32., 32.], [24., 32., 32.]],
                ],
            ]),
            bias: TestTensor::from_floats([32., 32.]),
        };
        test.assert_grads(grads);
    }

    #[test]
    fn test_conv2d_different_stride() {
        let test = Conv2dTestCase {
            batch_size: 2,
            channels_in: 2,
            channels_out: 2,
            kernel_size_1: 3,
            kernel_size_2: 3,
            padding_1: 1,
            padding_2: 1,
            stride_1: 3,
            stride_2: 1,
            height: 8,
            width: 8,
        };
        let grads = Grads {
            x: TestTensor::from_floats([
                [
                    [
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                    ],
                    [
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                    ],
                ],
                [
                    [
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                    ],
                    [
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                        [4., 6., 6., 6., 6., 6., 6., 4.],
                    ],
                ],
            ]),
            weight: TestTensor::from_floats([
                [
                    [[28., 32., 28.], [42., 48., 42.], [42., 48., 42.]],
                    [[28., 32., 28.], [42., 48., 42.], [42., 48., 42.]],
                ],
                [
                    [[28., 32., 28.], [42., 48., 42.], [42., 48., 42.]],
                    [[28., 32., 28.], [42., 48., 42.], [42., 48., 42.]],
                ],
            ]),
            bias: TestTensor::from_floats([48., 48.]),
        };
        test.assert_grads(grads);
    }

    struct Conv2dTestCase {
        batch_size: usize,
        channels_in: usize,
        channels_out: usize,
        kernel_size_1: usize,
        kernel_size_2: usize,
        padding_1: usize,
        padding_2: usize,
        stride_1: usize,
        stride_2: usize,
        height: usize,
        width: usize,
    }

    struct Grads {
        x: TestTensor<4>,
        weight: TestTensor<4>,
        bias: TestTensor<1>,
    }

    impl Conv2dTestCase {
        fn assert_grads(self, expected_grads: Grads) {
            let weight = TestADTensor::ones([
                self.channels_out,
                self.channels_in,
                self.kernel_size_1,
                self.kernel_size_2,
            ])
            .require_grad();
            let bias = TestADTensor::ones([self.channels_out]).require_grad();
            let x =
                TestADTensor::ones([self.batch_size, self.channels_in, self.height, self.width])
                    .require_grad();
            let output = conv2d(
                x.clone(),
                weight.clone(),
                Some(bias.clone()),
                [self.stride_1, self.stride_2],
                [self.padding_1, self.padding_2],
            );
            let grads = output.backward();

            // Assert
            let x_grad_actual = x.grad(&grads).unwrap();
            let weight_grad_actual = weight.grad(&grads).unwrap();
            let bias_grad_actual = bias.grad(&grads).unwrap();

            expected_grads
                .bias
                .to_data()
                .assert_approx_eq(&bias_grad_actual.to_data(), 3);
            expected_grads
                .x
                .to_data()
                .assert_approx_eq(&x_grad_actual.to_data(), 3);
            expected_grads
                .weight
                .to_data()
                .assert_approx_eq(&weight_grad_actual.to_data(), 3);
        }
    }
}
