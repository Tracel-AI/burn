use core::marker::PhantomData;

use super::base::MultiBackendBridge;
use crate::{
    repr::{ReprBackend, TensorHandle},
    router::{MultiDevice2, TensorHandle2},
    Shape,
};

pub struct ByteBridge<Backends> {
    backends: PhantomData<Backends>,
}

// TODO: refactor w/ visitor?
// pub trait BackendSwitchVisitor<B1: Backend> {
//     fn from_backend<B2: Backend>(handle: B2::FloatTensorPrimitive) -> B1::FloatTensorPrimitive;
// }

// Concrete implementation for bridge between two backends.
impl<B1: ReprBackend, B2: ReprBackend> MultiBackendBridge for ByteBridge<(B1, B2)> {
    type TensorHandle = TensorHandle2<B1, B2>;
    type Device = MultiDevice2<B1, B2>;

    fn change_backend_float(
        tensor: Self::TensorHandle,
        shape: Shape,
        device: &Self::Device,
    ) -> Self::TensorHandle {
        let msg = "Failed to read tensor data synchronously.
This can happen on platforms that don't support blocking futures like WASM.";
        match tensor {
            TensorHandle2::Handle1(handle) => match device {
                MultiDevice2::Device1(device) => {
                    // Same backend
                    let tensor = B1::float_tensor(TensorHandle { handle, shape });
                    let tensor = B1::float_to_device(tensor, device);
                    let handle = B1::float_tensor_handle(tensor);
                    TensorHandle2::Handle1(handle)
                }
                MultiDevice2::Device2(device) => {
                    let tensor = B1::float_tensor(TensorHandle { handle, shape });
                    let data = crate::try_read_sync(B1::float_into_data(tensor)).expect(msg);
                    let tensor = B2::float_from_data(data, device);
                    let handle = B2::float_tensor_handle(tensor);
                    TensorHandle2::Handle2(handle)
                }
            },
            TensorHandle2::Handle2(handle) => match device {
                MultiDevice2::Device1(device) => {
                    let tensor = B2::float_tensor(TensorHandle { handle, shape });
                    let data = crate::try_read_sync(B2::float_into_data(tensor)).expect(msg);
                    let tensor = B1::float_from_data(data, device);
                    let handle = B1::float_tensor_handle(tensor);
                    TensorHandle2::Handle1(handle)
                }
                MultiDevice2::Device2(device) => {
                    // Same backend
                    let tensor = B2::float_tensor(TensorHandle { handle, shape });
                    let tensor = B2::float_to_device(tensor, device);
                    let handle = B2::float_tensor_handle(tensor);
                    TensorHandle2::Handle2(handle)
                }
            },
        }
    }
}
