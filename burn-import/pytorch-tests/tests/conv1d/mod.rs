use burn::{
    module::Module,
    nn::conv::{Conv1d, Conv1dConfig},
    tensor::{backend::Backend, Tensor},
};

#[derive(Module, Debug)]
struct Net<B: Backend> {
    conv1: Conv1d<B>,
    conv2: Conv1d<B>,
}

impl<B: Backend> Net<B> {
    /// Create a new model from the given record.
    pub fn new_with(record: NetRecord<B>) -> Self {
        let conv1 = Conv1dConfig::new(2, 2, 2).init_with(record.conv1);
        let conv2 = Conv1dConfig::new(2, 2, 2)
            .with_bias(false)
            .init_with(record.conv2);
        Self { conv1, conv2 }
    }

    /// Forward pass of the model.
    pub fn forward(&self, x: Tensor<B, 3>) -> Tensor<B, 3> {
        let x = self.conv1.forward(x);

        self.conv2.forward(x)
    }
}

#[cfg(test)]
mod tests {
    type Backend = burn_ndarray::NdArray<f32>;
    use burn::record::{FullPrecisionSettings, HalfPrecisionSettings, Recorder};
    use burn_import::pytorch::PyTorchFileRecorder;

    use super::*;

    fn conv1d(record: NetRecord<Backend>, precision: usize) {
        let device = Default::default();

        let model = Net::<Backend>::new_with(record);

        let input = Tensor::<Backend, 3>::from_data(
            [[
                [
                    0.93708336, 0.65559506, 0.31379688, 0.19801933, 0.41619217, 0.28432965,
                ],
                [
                    0.33977574,
                    0.523_940_8,
                    0.798_063_9,
                    0.77176833,
                    0.01122457,
                    0.80996025,
                ],
            ]],
            &device,
        );

        let output = model.forward(input);

        let expected = Tensor::<Backend, 3>::from_data(
            [[
                [0.02987457, 0.03134188, 0.04234261, -0.02437721],
                [-0.03788019, -0.02972012, -0.00806090, -0.01981254],
            ]],
            &device,
        );

        output
            .to_data()
            .assert_approx_eq(&expected.to_data(), precision);
    }

    #[test]
    fn conv1d_full_precision() {
        let record = PyTorchFileRecorder::<FullPrecisionSettings>::default()
            .load("tests/conv1d/conv1d.pt".into())
            .expect("Failed to decode state");

        conv1d(record, 7);
    }

    #[test]
    fn conv1d_half_precision() {
        let record = PyTorchFileRecorder::<HalfPrecisionSettings>::default()
            .load("tests/conv1d/conv1d.pt".into())
            .expect("Failed to decode state");

        conv1d(record, 4);
    }
}
