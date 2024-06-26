use crate as burn_cube;
use burn_cube::prelude::*;

#[cube(launch)]
pub fn kernel_with_generics<F: Float>(mut output: Array<F>) {
    if UNIT_POS == UInt::new(0) {
        output[0] = F::new(5.0);
    }
}

#[cube(launch)]
pub fn kernel_without_generics(mut output: Array<F32>) {
    if UNIT_POS == UInt::new(0) {
        output[0] = F32::new(5.0);
    }
}

pub fn test_kernel_with_generics<R: Runtime>(client: ComputeClient<R::Server, R::Channel>) {
    let handle = client.create(f32::as_bytes(&[0.0, 1.0]));

    kernel_with_generics_launch::<F32, R>(
        client.clone(),
        CubeCount::new(1, 1, 1),
        KernelSettings::default(),
        ArrayHandle::new(&handle, 2),
    );

    let actual = client.read(handle.binding()).read_sync().unwrap();
    let actual = f32::from_bytes(&actual);

    assert_eq!(actual[0], 5.0);
}

pub fn test_kernel_without_generics<R: Runtime>(client: ComputeClient<R::Server, R::Channel>) {
    let handle = client.create(f32::as_bytes(&[0.0, 1.0]));

    kernel_without_generics_launch::<R>(
        client.clone(),
        CubeCount::new(1, 1, 1),
        KernelSettings::default(),
        ArrayHandle::new(&handle, 2),
    );

    let actual = client.read(handle.binding()).read_sync().unwrap();
    let actual = f32::from_bytes(&actual);

    assert_eq!(actual[0], 5.0);
}

#[allow(missing_docs)]
#[macro_export]
macro_rules! testgen_launch {
    () => {
        use super::*;

        #[test]
        fn test_launch_with_generics() {
            let client = TestRuntime::client(&Default::default());
            burn_cube::runtime_tests::launch::test_kernel_with_generics::<TestRuntime>(client);
        }

        #[test]
        fn test_launch_without_generics() {
            let client = TestRuntime::client(&Default::default());
            burn_cube::runtime_tests::launch::test_kernel_without_generics::<TestRuntime>(client);
        }
    };
}
