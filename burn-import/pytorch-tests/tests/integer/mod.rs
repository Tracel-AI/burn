use burn::{
    module::{Module, Param},
    tensor::{backend::Backend, Int, Tensor},
};

#[derive(Module, Debug)]
struct Net<B: Backend> {
    buffer: Param<Tensor<B, 1, Int>>,
}

impl<B: Backend> Net<B> {
    /// Create a new model from the given record.
    pub fn new_with(record: NetRecord<B>) -> Self {
        Self {
            buffer: record.buffer,
        }
    }

    /// Forward pass of the model.
    pub fn forward(&self, _x: Tensor<B, 2>) -> Tensor<B, 1, Int> {
        self.buffer.val()
    }
}

#[cfg(test)]
mod tests {
    type Backend = burn_ndarray::NdArray<f32>;

    use std::{env, path::Path};

    use burn::{
        record::{FullPrecisionSettings, NamedMpkFileRecorder, Recorder},
        tensor::Data,
    };

    use super::*;

    #[test]
    fn integer() {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let file_path = Path::new(&out_dir).join("model/integer");

        let record = NamedMpkFileRecorder::<FullPrecisionSettings>::default()
            .load(file_path)
            .expect("Failed to decode state");

        let model = Net::<Backend>::new_with(record);

        let input = Tensor::<Backend, 2>::ones([3, 3]);

        let output = model.forward(input);

        let expected = Tensor::<Backend, 1, Int>::from_data(Data::from([1, 2, 3]));

        assert_eq!(output.to_data(), expected.to_data());
    }
}
