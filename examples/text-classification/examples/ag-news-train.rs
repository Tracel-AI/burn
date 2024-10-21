use burn::{
    nn::transformer::TransformerEncoderConfig,
    optim::{decay::WeightDecayConfig, AdamConfig},
    tensor::backend::AutodiffBackend,
};

use text_classification::{training::ExperimentConfig, AgNewsDataset};

#[cfg(not(feature = "f16"))]
#[allow(dead_code)]
type ElemType = f32;
#[cfg(feature = "f16")]
type ElemType = burn::tensor::f16;

pub fn launch<B: AutodiffBackend>(devices: Vec<B::Device>) {
    let config = ExperimentConfig::new(
        TransformerEncoderConfig::new(256, 1024, 8, 4)
            .with_norm_first(true)
            .with_quiet_softmax(true),
        AdamConfig::new().with_weight_decay(Some(WeightDecayConfig::new(5e-5))),
    );

    text_classification::training::train::<B, AgNewsDataset>(
        devices,
        AgNewsDataset::train(),
        AgNewsDataset::test(),
        config,
        "/tmp/text-classification-ag-news",
    );
}

#[cfg(any(
    feature = "ndarray",
    feature = "ndarray-blas-netlib",
    feature = "ndarray-blas-openblas",
    feature = "ndarray-blas-accelerate",
))]
mod ndarray {
    use burn::backend::{
        ndarray::{NdArray, NdArrayDevice},
        Autodiff,
    };

    use crate::{launch, ElemType};

    pub fn run() {
        launch::<Autodiff<NdArray<ElemType>>>(vec![NdArrayDevice::Cpu]);
    }
}

#[cfg(feature = "libtorch-gpu")]
mod libtorch_gpu {
    use burn::backend::{
        libtorch::{LibTorch, LibTorchDevice},
        Autodiff,
    };

    use crate::{launch, ElemType};

    pub fn run() {
        #[cfg(not(target_os = "macos"))]
        let device = LibTorchDevice::Cuda(0);
        #[cfg(target_os = "macos")]
        let device = LibTorchDevice::Mps;

        launch::<Autodiff<LibTorch<ElemType>>>(vec![device]);
    }
}

#[cfg(feature = "libtorch-cpu")]
mod libtorch_cpu {
    use burn::backend::{
        libtorch::{LibTorch, LibTorchDevice},
        Autodiff,
    };

    use crate::{launch, ElemType};

    pub fn run() {
        launch::<Autodiff<LibTorch<ElemType>>>(vec![LibTorchDevice::Cpu]);
    }
}

#[cfg(feature = "wgpu")]
mod wgpu {
    use crate::{launch, ElemType};
    use burn::backend::{wgpu::Wgpu, Autodiff};

    pub fn run() {
        launch::<Autodiff<Wgpu<ElemType, i32>>>(vec![Default::default()]);
    }
}

#[cfg(feature = "cuda")]
mod cuda {
    use crate::{launch, ElemType};
    use burn::backend::{Autodiff, Cuda};

    pub fn run() {
        launch::<Autodiff<Cuda<ElemType, i32>>>(vec![Default::default()]);
    }
}

fn main() {
    #[cfg(any(
        feature = "ndarray",
        feature = "ndarray-blas-netlib",
        feature = "ndarray-blas-openblas",
        feature = "ndarray-blas-accelerate",
    ))]
    ndarray::run();
    #[cfg(feature = "libtorch-gpu")]
    libtorch_gpu::run();
    #[cfg(feature = "libtorch-cpu")]
    libtorch_cpu::run();
    #[cfg(feature = "wgpu")]
    wgpu::run();
    #[cfg(feature = "cuda")]
    cuda::run();
}
