pub trait GraphicsAPI: Send + Sync + core::fmt::Debug + Default + Clone {
    fn id() -> u32;
    fn backend() -> wgpu::Backend;
}

#[derive(Default, Debug, Clone)]
pub struct Vulkan;
#[derive(Default, Debug, Clone)]
pub struct Metal;
#[derive(Default, Debug, Clone)]
pub struct OpenGL;
#[derive(Default, Debug, Clone)]
pub struct Dx11;
#[derive(Default, Debug, Clone)]
pub struct Dx12;
#[derive(Default, Debug, Clone)]
pub struct WebGPU;

impl GraphicsAPI for Vulkan {
    fn id() -> u32 {
        1
    }
    fn backend() -> wgpu::Backend {
        wgpu::Backend::Vulkan
    }
}

impl GraphicsAPI for Metal {
    fn id() -> u32 {
        2
    }
    fn backend() -> wgpu::Backend {
        wgpu::Backend::Metal
    }
}

impl GraphicsAPI for OpenGL {
    fn id() -> u32 {
        3
    }
    fn backend() -> wgpu::Backend {
        wgpu::Backend::Gl
    }
}

impl GraphicsAPI for Dx11 {
    fn id() -> u32 {
        4
    }
    fn backend() -> wgpu::Backend {
        wgpu::Backend::Dx11
    }
}

impl GraphicsAPI for Dx12 {
    fn id() -> u32 {
        5
    }
    fn backend() -> wgpu::Backend {
        wgpu::Backend::Dx12
    }
}

impl GraphicsAPI for WebGPU {
    fn id() -> u32 {
        6
    }
    fn backend() -> wgpu::Backend {
        wgpu::Backend::BrowserWebGpu
    }
}
