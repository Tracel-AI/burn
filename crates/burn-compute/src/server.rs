use crate::{
    memory_management::{MemoryManagement, MemoryTensorBufHandle},
    storage::ComputeStorage,
    tune::AutotuneKey,
};
use alloc::vec::Vec;
use burn_common::reader::Reader;
use core::fmt::Debug;

/// The compute server is responsible for handling resources and computations over resources.
///
/// Everything in the server is mutable, therefore it should be solely accessed through the
/// [compute channel](crate::channel::ComputeChannel) for thread safety.
pub trait ComputeServer: Send + core::fmt::Debug
where
    Self: Sized,
{
    /// The kernel type defines the computation algorithms.
    type Kernel: Send;
    /// The [storage](ComputeStorage) type defines how data is stored and accessed.
    type Storage: ComputeStorage;
    /// The [memory management](MemoryManagement) type defines strategies for allocation in the [storage](ComputeStorage) type.
    type MemoryManagement: MemoryManagement<Self::Storage>;
    /// The key used to cache operations used on specific inputs in autotune
    type AutotuneKey: AutotuneKey;

    /// Given a handle, returns the owned resource as bytes.
    fn read(&mut self, handle: BufHandle<Self>) -> Reader<Vec<u8>>;

    /// Given a resource as bytes, stores it and returns the memory handle.
    fn create(&mut self, data: &[u8]) -> TensorBufHandle<Self>;

    /// Reserves `size` bytes in the storage, and returns a handle over them.
    fn empty(&mut self, size: usize) -> TensorBufHandle<Self>;

    /// Executes the `kernel` over the given memory `handles`.
    ///
    /// Kernels have mutable access to every resource they are given
    /// and are responsible of determining which should be read or written.
    fn execute(&mut self, kernel: Self::Kernel, handles: Vec<BufHandle<Self>>);

    /// Wait for the completion of every task in the server.
    fn sync(&mut self);
}

/// Server handle containing the [memory handle](MemoryManagement::Handle).
#[derive(new, Debug)]
pub struct TensorBufHandle<Server: ComputeServer> {
    /// Handle for the memory in use.
    pub memory: <Server::MemoryManagement as MemoryManagement<Server::Storage>>::TensorBufHandle,
}

/// Server handle containing the [memory handle](MemoryManagement::Handle).
#[derive(new)]
pub struct BufHandle<Server: ComputeServer> {
    /// Handle for the memory in use.
    pub memory: <Server::MemoryManagement as MemoryManagement<Server::Storage>>::BufHandle,
}

impl<Server: ComputeServer> TensorBufHandle<Server> {
    /// If the tensor handle can be reused inplace.
    pub fn can_mut(&self) -> bool {
        MemoryTensorBufHandle::can_mut(&self.memory)
    }
}

impl<Server: ComputeServer> TensorBufHandle<Server> {
    /// Disconnect the buffer from the tensor.
    ///
    /// The handle can then be sent to the compute server.
    pub fn disconnect(&self) -> BufHandle<Server> {
        BufHandle {
            memory: MemoryTensorBufHandle::disconnect(&self.memory),
        }
    }
}

impl<Server: ComputeServer> Clone for TensorBufHandle<Server> {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
        }
    }
}

impl<Server: ComputeServer> Clone for BufHandle<Server> {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
        }
    }
}
