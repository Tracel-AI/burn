use crate::{
    memory_id_type,
    storage::{ComputeStorage, StorageHandle, StorageUtilization},
};
use alloc::vec::Vec;
use hashbrown::HashMap;

#[cfg(all(not(target_family = "wasm"), feature = "std"))]
use std::time;
#[cfg(all(target_family = "wasm", feature = "std"))]
use web_time as time;

// The ChunkId allows to keep track of how many references there are to a specific chunk.
memory_id_type!(ChunkId, ChunkTensorBufHandle, ChunkBufHandle);
// The SliceId allows to keep track of how many references there are to a specific slice.
memory_id_type!(SliceId, SliceTensorBufHandle, SliceBufHandle);

/// A tensor memory handle, referring to either a chunk or a slice.
#[derive(Debug, Clone)]
pub enum SimpleTensorBufHandle {
    /// A whole chunk of memory.
    Chunk(ChunkTensorBufHandle),
    /// A slice of a chunk of memory.
    Slice(SliceTensorBufHandle),
}

/// A memory handle, referring to either a chunk or a slice.
#[derive(Debug, Clone)]
pub enum SimpleBufHandle {
    /// A whole chunk of memory.
    Chunk(ChunkBufHandle),
    /// A slice of a chunk of memory.
    Slice(SliceBufHandle),
}

/// The strategy defines the frequency at which deallocation of unused memory chunks should occur.
#[derive(Debug)]
pub enum DeallocStrategy {
    /// Once every n calls to reserve.
    PeriodTick {
        /// Number of calls to be executed before triggering the deallocation.
        period: usize,
        /// Current state. Should start at zero.
        state: usize,
    },
    #[cfg(feature = "std")]
    /// Once every period of time
    PeriodTime {
        /// Number of time before triggering the deallocation.
        period: time::Duration,
        /// Current state. Should start at now.
        state: time::Instant,
    },
    /// Never deallocate.
    Never,
}

/// The strategy defines when to reuse chunk with slices.
#[derive(Debug)]
pub enum SliceStrategy {
    /// Never use slices.
    Never,
    /// Ratio needed before the chunk can be used as a slice. Between 0 and 1.
    Ratio(f32),
    /// When the reserved memory is at least {} bytes.
    MinimumSize(usize),
    /// When the reserved memory less than {} bytes.
    MaximumSize(usize),
}

impl SliceStrategy {
    /// If the chunk can be used with a slice.
    pub fn can_use_chunk(&self, chunk_size: usize, reserved_size: usize) -> bool {
        if chunk_size < reserved_size {
            return false;
        }

        match self {
            SliceStrategy::Never => false,
            SliceStrategy::Ratio(ratio) => (reserved_size as f32 / chunk_size as f32) >= *ratio,
            SliceStrategy::MinimumSize(bytes) => reserved_size >= *bytes,
            SliceStrategy::MaximumSize(bytes) => reserved_size <= *bytes,
        }
    }
}

impl DeallocStrategy {
    /// Create a new strategy with the given period.
    pub fn new_period_tick(period: usize) -> Self {
        DeallocStrategy::PeriodTick { period, state: 0 }
    }

    fn should_dealloc(&mut self) -> bool {
        match self {
            DeallocStrategy::PeriodTick { period, state } => {
                *state = (*state + 1) % *period;
                *state == 0
            }
            #[cfg(feature = "std")]
            DeallocStrategy::PeriodTime { period, state } => {
                if &state.elapsed() > period {
                    *state = time::Instant::now();
                    true
                } else {
                    false
                }
            }
            DeallocStrategy::Never => false,
        }
    }
}

#[derive(new)]
struct Chunk {
    storage: StorageHandle,
    handle: ChunkTensorBufHandle,
    slices: Vec<SliceId>,
}

#[derive(new)]
struct Slice {
    storage: StorageHandle,
    handle: SliceTensorBufHandle,
    // It is important to keep the chunk handle inside the slice, since it increases the ref count
    // on the chunk id and make the `is_free` method returns false until the slice is freed.
    //
    // TL;DR we can't only store the chunk id.
    chunk: ChunkTensorBufHandle,
}

/// Reserves and keeps track of chunks of memory in the storage, and slices upon these chunks.
pub struct SimpleMemoryManagement<Storage> {
    chunks: HashMap<ChunkId, Chunk>,
    slices: HashMap<SliceId, Slice>,
    dealloc_strategy: DeallocStrategy,
    slice_strategy: SliceStrategy,
    storage: Storage,
}

impl<Storage> core::fmt::Debug for SimpleMemoryManagement<Storage> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(
            alloc::format!(
                "SimpleMemoryManagement {:?} - {:?}",
                self.dealloc_strategy,
                core::any::type_name::<Storage>(),
            )
            .as_str(),
        )
    }
}

impl super::MemoryBufHandle for SimpleBufHandle {}

impl super::MemoryTensorBufHandle<SimpleBufHandle> for SimpleTensorBufHandle {
    /// Returns true if referenced by only one tensor, and only once by the
    /// memory management hashmaps
    fn can_mut(&self) -> bool {
        match &self {
            SimpleTensorBufHandle::Chunk(id) => id.can_mut(),
            SimpleTensorBufHandle::Slice(id) => id.can_mut(),
        }
    }

    fn disconnect(&self) -> SimpleBufHandle {
        match self {
            Self::Chunk(handle) => SimpleBufHandle::Chunk(handle.handle()),
            Self::Slice(handle) => SimpleBufHandle::Slice(handle.handle()),
        }
    }
}

impl<Storage: ComputeStorage> super::MemoryManagement<Storage> for SimpleMemoryManagement<Storage> {
    type TensorBufHandle = SimpleTensorBufHandle;
    type BufHandle = SimpleBufHandle;

    /// Returns the resource from the storage, for the specified handle.
    fn get(&mut self, handle: Self::BufHandle) -> Storage::Resource {
        let storage = match handle {
            SimpleBufHandle::Chunk(chunk) => {
                &self
                    .chunks
                    .get(chunk.id())
                    .expect("Storage found for the given execution buffer handle")
                    .storage
            }
            SimpleBufHandle::Slice(slice) => {
                &self
                    .slices
                    .get(slice.id())
                    .expect("Storage found for the given execution buffer handle")
                    .storage
            }
        };

        self.storage.get(storage)
    }

    /// Reserves memory of specified size using the reserve algorithm, and return
    /// a handle to the reserved memory.
    ///
    /// Also clean ups, removing unused slices, and chunks if permitted by deallocation strategy.
    fn reserve(&mut self, size: usize) -> Self::TensorBufHandle {
        self.cleanup_slices();

        let handle = self.reserve_algorithm(size);

        if self.dealloc_strategy.should_dealloc() {
            self.cleanup_chunks();
        }

        handle
    }

    fn alloc(&mut self, size: usize) -> Self::TensorBufHandle {
        self.create_chunk(size)
    }

    fn dealloc(&mut self, handle: Self::BufHandle) {
        match handle {
            SimpleBufHandle::Chunk(chunk) => {
                if let Some(chunk) = self.chunks.remove(chunk.id()) {
                    self.storage.dealloc(chunk.storage.id);
                }
            }
            SimpleBufHandle::Slice(_) => panic!("Can't dealloc slice manually"),
        }
    }

    fn storage(&mut self) -> &mut Storage {
        &mut self.storage
    }
}

impl<Storage: ComputeStorage> SimpleMemoryManagement<Storage> {
    /// Creates a new instance using the given storage, deallocation strategy and slice strategy.
    pub fn new(
        storage: Storage,
        dealloc_strategy: DeallocStrategy,
        slice_strategy: SliceStrategy,
    ) -> Self {
        Self {
            chunks: HashMap::new(),
            slices: HashMap::new(),
            dealloc_strategy,
            slice_strategy,
            storage,
        }
    }

    fn reserve_algorithm(&mut self, size: usize) -> SimpleTensorBufHandle {
        // Looks for a large enough, existing but unused chunk of memory.
        let chunk = self.find_free_chunk(size);

        match chunk {
            Some((chunk_id, chunk_size)) => {
                if size == chunk_size {
                    // If there is one of exactly the same size, it reuses it.
                    SimpleTensorBufHandle::Chunk(chunk_id.clone())
                } else {
                    // Otherwise creates a slice of the right size upon it, always starting at zero.
                    self.create_slice(size, chunk_id)
                }
            }
            // If no chunk available, creates one of exactly the right size.
            None => self.create_chunk(size),
        }
    }

    /// Finds the smallest of the free and large enough chunks to fit `size`
    /// Returns the chunk's id and size.
    fn find_free_chunk(&self, size: usize) -> Option<(ChunkTensorBufHandle, usize)> {
        let mut size_diff_current = usize::MAX;
        let mut current = None;

        for chunk in self.chunks.values() {
            // If chunk is already used, we do not choose it
            if !chunk.handle.is_free() {
                continue;
            }

            let storage_size = chunk.storage.size();

            // If we find a chunk of exactly the right size, we stop searching altogether
            if size == storage_size {
                current = Some(chunk);
                break;
            }

            // Finds the smallest of the large enough chunks that can accept a slice
            // of the given size
            if self.slice_strategy.can_use_chunk(storage_size, size) {
                let size_diff = storage_size - size;

                if size_diff < size_diff_current {
                    current = Some(chunk);
                    size_diff_current = size_diff;
                }
            }
        }

        current.map(|chunk| (chunk.handle.clone(), chunk.storage.size()))
    }

    /// Creates a slice of size `size` upon the given chunk.
    ///
    /// For now slices must start at zero, therefore there can be only one per chunk
    fn create_slice(&mut self, size: usize, handle: ChunkTensorBufHandle) -> SimpleTensorBufHandle {
        let chunk = self.chunks.get_mut(handle.id()).unwrap();
        let slice_handle = SliceTensorBufHandle::new();

        let storage = StorageHandle {
            id: chunk.storage.id.clone(),
            utilization: StorageUtilization::Slice(0, size),
        };

        if chunk.slices.is_empty() {
            self.slices.insert(
                *slice_handle.id(),
                Slice::new(storage, slice_handle.clone(), handle.clone()),
            );
        } else {
            panic!("Can't have more than 1 slice yet.");
        }

        chunk.slices.push(*slice_handle.id());

        SimpleTensorBufHandle::Slice(slice_handle)
    }

    /// Creates a chunk of given size by allocating on the storage.
    fn create_chunk(&mut self, size: usize) -> SimpleTensorBufHandle {
        let storage = self.storage.alloc(size);
        let handle = ChunkTensorBufHandle::new();

        self.chunks.insert(
            *handle.id(),
            Chunk::new(storage, handle.clone(), Vec::new()),
        );

        SimpleTensorBufHandle::Chunk(handle)
    }

    /// Deallocates free chunks and remove them from chunks map.
    fn cleanup_chunks(&mut self) {
        let mut ids_to_remove = Vec::new();

        self.chunks.iter().for_each(|(chunk_id, chunk)| {
            if chunk.handle.can_be_dealloc() {
                ids_to_remove.push(*chunk_id);
            }
        });

        ids_to_remove
            .iter()
            .map(|chunk_id| self.chunks.remove(chunk_id).unwrap())
            .for_each(|chunk| {
                self.storage.dealloc(chunk.storage.id);
            });
    }

    /// Removes free slices from slice map and corresponding chunks.
    fn cleanup_slices(&mut self) {
        let mut ids_to_remove = Vec::new();

        self.slices.iter().for_each(|(slice_id, slice)| {
            if slice.handle.is_free() {
                ids_to_remove.push(*slice_id);
            }
        });

        ids_to_remove
            .iter()
            .map(|slice_id| self.slices.remove(slice_id).unwrap())
            .for_each(|slice| {
                let chunk = self.chunks.get_mut(slice.chunk.id()).unwrap();
                chunk.slices.retain(|id| id != slice.handle.id());
            });
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        memory_management::{MemoryManagement, MemoryTensorBufHandle, SliceStrategy},
        storage::BytesStorage,
    };

    use super::{DeallocStrategy, SimpleMemoryManagement};

    #[test]
    fn can_mut_with_single_tensor_reference() {
        let mut memory_management = SimpleMemoryManagement::new(
            BytesStorage::default(),
            DeallocStrategy::Never,
            SliceStrategy::Never,
        );

        let chunk_size = 4;
        let simple_handle = memory_management.create_chunk(chunk_size);

        let x = simple_handle.clone();
        core::mem::drop(simple_handle);

        assert!(x.can_mut());
    }

    #[test]
    fn two_tensor_references_remove_mutability() {
        let mut memory_management = SimpleMemoryManagement::new(
            BytesStorage::default(),
            DeallocStrategy::Never,
            SliceStrategy::Never,
        );

        let chunk_size = 4;
        let simple_handle = memory_management.create_chunk(chunk_size);

        let x = simple_handle.clone();

        assert!(!simple_handle.can_mut());
        assert!(!x.can_mut())
    }

    #[test]
    fn when_non_empty_chunk_exists_and_other_one_created_there_should_be_two() {
        let mut memory_management = SimpleMemoryManagement::new(
            BytesStorage::default(),
            DeallocStrategy::Never,
            SliceStrategy::Never,
        );
        let chunk_size = 4;
        let _chunk_handle = memory_management.reserve(chunk_size);
        let _new_handle = memory_management.reserve(chunk_size);

        assert_eq!(memory_management.chunks.len(), 2);
    }

    #[test]
    fn when_empty_chunk_is_cleaned_upexists_it_disappears() {
        let mut memory_management = SimpleMemoryManagement::new(
            BytesStorage::default(),
            DeallocStrategy::Never,
            SliceStrategy::Never,
        );
        let chunk_size = 4;
        let chunk_handle = memory_management.reserve(chunk_size);
        drop(chunk_handle);
        memory_management.cleanup_chunks();

        assert_eq!(memory_management.chunks.len(), 0);
    }

    #[test]
    fn never_dealloc_strategy_never_deallocs() {
        let mut never_dealloc = DeallocStrategy::Never;
        for _ in 0..20 {
            assert!(!never_dealloc.should_dealloc())
        }
    }

    #[test]
    fn period_tick_dealloc_strategy_should_dealloc_after_period() {
        let period = 3;
        let mut period_tick_dealloc = DeallocStrategy::new_period_tick(period);

        for _ in 0..3 {
            for _ in 0..period - 1 {
                assert!(!period_tick_dealloc.should_dealloc());
            }
            assert!(period_tick_dealloc.should_dealloc());
        }
    }

    #[test]
    fn slice_strategy_minimum_bytes() {
        let strategy = SliceStrategy::MinimumSize(100);

        assert!(strategy.can_use_chunk(200, 101));
        assert!(!strategy.can_use_chunk(200, 99));
    }

    #[test]
    fn slice_strategy_maximum_bytes() {
        let strategy = SliceStrategy::MaximumSize(100);

        assert!(strategy.can_use_chunk(200, 99));
        assert!(!strategy.can_use_chunk(200, 101));
    }

    #[test]
    fn slice_strategy_ratio() {
        let strategy = SliceStrategy::Ratio(0.9);

        assert!(strategy.can_use_chunk(200, 180));
        assert!(!strategy.can_use_chunk(200, 179));
    }
}
