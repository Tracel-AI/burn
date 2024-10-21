pub mod burnbenchapp;
pub mod persistence;

/// Simple parse to retrieve additional argument passed to cargo bench command
/// We cannot use clap here as clap parser does not allow to have unknown arguments.
pub fn get_argument<'a>(args: &'a [String], arg_name: &'a str) -> Option<&'a str> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            arg if arg == arg_name && i + 1 < args.len() => {
                return Some(&args[i + 1]);
            }
            _ => i += 1,
        }
    }
    None
}

/// Specialized function to retrieve the sharing token
pub fn get_sharing_token(args: &[String]) -> Option<&str> {
    get_argument(args, "--sharing-token")
}

/// Specialized function to retrieve the sharing URL
pub fn get_sharing_url(args: &[String]) -> Option<&str> {
    get_argument(args, "--sharing-url")
}

#[macro_export]
macro_rules! bench_on_backend {
    () => {
        use std::env;
        let args: Vec<String> = env::args().collect();
        let url = backend_comparison::get_sharing_url(&args);
        let token = backend_comparison::get_sharing_token(&args);
        #[cfg(feature = "candle-accelerate")]
        let feature_name = "candle-accelerate";
        #[cfg(feature = "candle-cpu")]
        let feature_name = "candle-cpu";
        #[cfg(feature = "candle-cuda")]
        let feature_name = "candle-cuda";
        #[cfg(feature = "candle-metal")]
        let feature_name = "candle-metal";
        #[cfg(feature = "ndarray")]
        let feature_name = "ndarray";
        #[cfg(feature = "ndarray-blas-accelerate")]
        let feature_name = "ndarray-blas-accelerate";
        #[cfg(feature = "ndarray-blas-netlib")]
        let feature_name = "ndarray-blas-netlib";
        #[cfg(feature = "ndarray-blas-openblas")]
        let feature_name = "ndarray-blas-openblas";
        #[cfg(feature = "libtorch-cpu")]
        let feature_name = "libtorch-cpu";
        #[cfg(feature = "libtorch-gpu")]
        let feature_name = "libtorch-gpu";
        #[cfg(feature = "wgpu")]
        let feature_name = "wgpu";
        #[cfg(feature = "wgpu-fusion")]
        let feature_name = "wgpu-fusion";
        #[cfg(feature = "vulkan")]
        let feature_name = "vulkan";
        #[cfg(feature = "vulkan-fusion")]
        let feature_name = "vulkan-fusion";
        #[cfg(feature = "cuda")]
        let feature_name = "cuda";
        #[cfg(feature = "cuda-fusion")]
        let feature_name = "cuda-fusion";

        #[cfg(any(feature = "wgpu"))]
        {
            use burn::backend::wgpu::{Wgpu, WgpuDevice};

            bench::<Wgpu<f32, i32>>(&WgpuDevice::default(), feature_name, url, token);
        }

        #[cfg(any(feature = "vulkan"))]
        {
            use burn::backend::vulkan::{Vulkan, WgpuDevice};

            bench::<Vulkan<half::f16, i32>>(&WgpuDevice::default(), feature_name, url, token);
        }

        #[cfg(feature = "libtorch-gpu")]
        {
            use burn::backend::{libtorch::LibTorchDevice, LibTorch};

            #[cfg(not(target_os = "macos"))]
            let device = LibTorchDevice::Cuda(0);
            #[cfg(target_os = "macos")]
            let device = LibTorchDevice::Mps;
            bench::<LibTorch<half::f16>>(&device, feature_name, url, token);
        }

        #[cfg(feature = "libtorch-cpu")]
        {
            use burn::backend::{libtorch::LibTorchDevice, LibTorch};

            let device = LibTorchDevice::Cpu;
            bench::<LibTorch>(&device, feature_name, url, token);
        }

        #[cfg(any(
            feature = "ndarray",
            feature = "ndarray-blas-netlib",
            feature = "ndarray-blas-openblas",
            feature = "ndarray-blas-accelerate",
        ))]
        {
            use burn::backend::ndarray::NdArrayDevice;
            use burn::backend::NdArray;

            let device = NdArrayDevice::Cpu;
            bench::<NdArray>(&device, feature_name, url, token);
        }

        #[cfg(feature = "candle-cpu")]
        {
            use burn::backend::candle::CandleDevice;
            use burn::backend::Candle;

            let device = CandleDevice::Cpu;
            bench::<Candle>(&device, feature_name, url, token);
        }

        #[cfg(feature = "candle-cuda")]
        {
            use burn::backend::candle::CandleDevice;
            use burn::backend::Candle;

            let device = CandleDevice::cuda(0);
            bench::<Candle>(&device, feature_name, url, token);
        }

        #[cfg(feature = "candle-metal")]
        {
            use burn::backend::candle::CandleDevice;
            use burn::backend::Candle;

            let device = CandleDevice::metal(0);
            bench::<Candle>(&device, feature_name, url, token);
        }

        #[cfg(feature = "cuda")]
        {
            use burn::backend::cuda::{Cuda, CudaDevice};

            bench::<Cuda<half::f16>>(&CudaDevice::default(), feature_name, url, token);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::sharing_token_argument_with_value(&["--sharing-token", "token123"], Some("token123"))]
    #[case::sharing_token_argument_no_value(&["--sharing-token"], None)]
    #[case::sharing_token_argument_with_additional_arguments(&["--other-arg", "value", "--sharing-token", "token789"], Some("token789"))]
    #[case::other_argument(&["--other-arg", "value"], None)]
    #[case::no_argument(&[], None)]
    fn test_get_sharing_token(#[case] args: &[&str], #[case] expected: Option<&str>) {
        let args = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        assert_eq!(get_sharing_token(&args), expected);
    }

    #[rstest]
    #[case::sharing_url_argument_with_value(&["--sharing-url", "url123"], Some("url123"))]
    #[case::sharing_url_argument_no_value(&["--sharing-url"], None)]
    #[case::sharing_url_argument_with_additional_arguments(&["--other-arg", "value", "--sharing-url", "url789"], Some("url789"))]
    #[case::other_argument(&["--other-arg", "value"], None)]
    #[case::no_argument(&[], None)]
    fn test_get_sharing_url(#[case] args: &[&str], #[case] expected: Option<&str>) {
        let args = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        assert_eq!(get_sharing_url(&args), expected);
    }
}
