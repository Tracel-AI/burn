#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! Burn ndarray backend.

#[macro_use]
extern crate derive_new;

#[cfg(any(
    feature = "blas-netlib",
    feature = "blas-openblas",
    feature = "blas-openblas-system",
))]
extern crate blas_src;

mod backend;
mod element;
mod ops;
mod parallel;
mod sharing;
mod tensor;

pub use backend::*;
pub use element::FloatNdArrayElement;
pub(crate) use sharing::*;
pub use tensor::*;

extern crate alloc;

#[cfg(test)]
mod tests {
    use burn_tensor::{DynData, backend::Backend};

    type TestBackend = crate::NdArray<f32>;
    type TestTensor<const D: usize> = burn_tensor::Tensor<TestBackend, D>;
    type TestTensorInt<const D: usize> = burn_tensor::Tensor<TestBackend, D, burn_tensor::Int>;
    type TestTensorBool<const D: usize> = burn_tensor::Tensor<TestBackend, D, burn_tensor::Bool>;
    pub type TestTensorDyn = burn_tensor::DynTensor<<TestBackend as Backend>::DynTensorPrimitive>;

    use alloc::format;
    use alloc::vec;

    burn_tensor::testgen_all!();

    #[cfg(feature = "std")]
    burn_autodiff::testgen_all!();
}
