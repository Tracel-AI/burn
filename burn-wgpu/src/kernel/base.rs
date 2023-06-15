use super::SourceTemplate;
use crate::{element::WgpuElement, tensor::WgpuTensor};
use std::marker::PhantomData;

/// Generate wgpu kernel source code to create [compute shader modules](wgpu::ShaderModule).
pub trait StaticKernelGenerator: 'static {
    /// Generate the source code.
    fn source() -> SourceTemplate;
}

/// Generate wgpu kernel source code to create [compute shader modules](wgpu::ShaderModule).
pub trait DynamicKernelGenerator {
    /// Generate the source code.
    fn source(self) -> SourceTemplate;
    fn id(&self) -> String;
}

#[macro_export]
macro_rules! kernel_wgsl {
    (
        $struct:ident,
        $file:expr
    ) => {
        #[derive(new)]
        pub struct $struct;

        impl $crate::kernel::StaticKernelGenerator for $struct {
            fn source() -> $crate::kernel::SourceTemplate {
                $crate::kernel::SourceTemplate::new(include_str!($file))
            }
        }
    };
}

/// Generate kernel source code by replacing some information using templating.
pub struct KernelSettings<
    K: StaticKernelGenerator,
    E: WgpuElement,
    I: WgpuElement,
    const WORKGROUP_X_SIZE: usize,
    const WORKGROUP_Y_SIZE: usize,
    const WORKGROUP_Z_SIZE: usize,
> {
    _k: PhantomData<K>,
    _e: PhantomData<E>,
    _i: PhantomData<I>,
}

impl<
        K: StaticKernelGenerator,
        E: WgpuElement,
        I: WgpuElement,
        const WORKGROUP_X_SIZE: usize,
        const WORKGROUP_Y_SIZE: usize,
        const WORKGROUP_Z_SIZE: usize,
    > StaticKernelGenerator
    for KernelSettings<K, E, I, WORKGROUP_X_SIZE, WORKGROUP_Y_SIZE, WORKGROUP_Z_SIZE>
{
    fn source() -> SourceTemplate {
        K::source()
            .register("workgroup_size_x", WORKGROUP_X_SIZE.to_string())
            .register("workgroup_size_y", WORKGROUP_Y_SIZE.to_string())
            .register("workgroup_size_z", WORKGROUP_Z_SIZE.to_string())
            .register("elem", E::type_name())
            .register("int", I::type_name())
    }
}

/// Generate kernel source code by replacing some information using templating.
#[derive(new)]
pub struct DynamicKernelSettings<K: StaticKernelGenerator, E: WgpuElement, I: WgpuElement> {
    workgroup_x_size: usize,
    workgroup_y_size: usize,
    workgroup_z_size: usize,
    _k: PhantomData<K>,
    _e: PhantomData<E>,
    _i: PhantomData<I>,
}

impl<K: StaticKernelGenerator, E: WgpuElement, I: WgpuElement> DynamicKernelGenerator
    for DynamicKernelSettings<K, E, I>
{
    fn source(self) -> SourceTemplate {
        K::source()
            .register("workgroup_size_x", self.workgroup_x_size.to_string())
            .register("workgroup_size_y", self.workgroup_y_size.to_string())
            .register("workgroup_size_z", self.workgroup_z_size.to_string())
            .register("elem", E::type_name())
            .register("int", I::type_name())
    }

    fn id(&self) -> String {
        let id = core::any::TypeId::of::<K>();

        format!(
            "{:?}-dyn-settings{}-{}-{}",
            id, self.workgroup_x_size, self.workgroup_y_size, self.workgroup_z_size
        )
    }
}

/// Create a vector containing the dimension, strides and shape of tensors.
///
/// # Example
///
/// With two tensors (lhs, rhs)
///
/// | Indexes                  | Value       |
/// |:------------------------:|:-----------:|
/// |           0..1           | D           |
/// |           1..D + 1       | lhs strides |
/// |     (D + 1)..(2 * D + 1) | rhs strides |
/// | (2 * D + 1)..(3 * D + 1) | lhs shape   |
/// | (3 * D + 1)..(4 * D + 1) | rhs shape   |
pub(crate) fn build_info<E: WgpuElement, const D: usize>(
    tensors: &[&WgpuTensor<E, D>],
) -> Vec<u32> {
    let mut info: Vec<u32> = vec![0; tensors.len() * 2 * D + 1];
    info[0] = D as u32;

    let mut current = 1;
    for tensor in tensors.iter() {
        for d in 0..D {
            info[current] = tensor.strides[d] as u32;
            current += 1;
        }
    }
    for tensor in tensors.iter() {
        for d in 0..D {
            info[current] = tensor.shape.dims[d] as u32;
            current += 1;
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::any::TypeId;

    #[test]
    fn test_kernel_type_id() {
        kernel_wgsl!(Add, "../template/binary_elemwise.wgsl");

        let type_id_1 = TypeId::of::<KernelSettings<Add, f32, i32, 2, 3, 4>>();
        let type_id_2 = TypeId::of::<KernelSettings<Add, f32, i32, 2, 3, 5>>();
        let type_id_3 = TypeId::of::<KernelSettings<Add, f32, i32, 2, 3, 4>>();

        assert_ne!(type_id_1, type_id_2);
        assert_eq!(type_id_1, type_id_3);
    }
}
