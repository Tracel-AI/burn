pub use crate::tensor::backend::Backend;

pub mod ad {
    pub use crate::tensor::backend::ADBackend as Backend;
    #[cfg(feature = "tch")]
    pub type Tch<E> = crate::tensor::backend::autodiff::ADBackendTch<E>;
    #[cfg(feature = "ndarray")]
    pub type NdArray<E> = crate::tensor::backend::autodiff::ADBackendNdArray<E>;
}

#[cfg(feature = "tch")]
pub type Tch<E> = crate::tensor::backend::tch::TchBackend<E>;
#[cfg(feature = "ndarray")]
pub type NdArray<E> = crate::tensor::backend::ndarray::NdArrayBackend<E>;
