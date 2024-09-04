use uuid::Uuid;

const CHUNK_SIZE: usize = 64_000;
type ChunkMemory = [u8; CHUNK_SIZE];

#[derive(Debug)]
pub struct Chunk {
    id: Uuid,
    memory: Box<ChunkMemory>,
}

impl Chunk {
    pub const SIZE: usize = CHUNK_SIZE;

    pub fn new_zeroed(id: Uuid) -> Self {
        use std::alloc::Layout;

        const MEMORY_LAYOUT: Layout = Layout::new::<ChunkMemory>();

        // SAFETY: Allocate a zero-initialized `ChunkMemory`-sized region of memory, and read it into box holding a `ChunkMemory`.
        unsafe {
            let memory = std::alloc::alloc_zeroed(MEMORY_LAYOUT);
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(memory, MEMORY_LAYOUT.size());
            let boxed_slice = Box::from_raw(slice_ptr);
            let boxed_array = boxed_slice.try_into().unwrap_unchecked();

            Self {
                id,
                memory: boxed_array,
            }
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn into_box(self) -> Box<[u8]> {
        self.memory
    }
}

impl core::ops::Deref for Chunk {
    type Target = ChunkMemory;

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl core::ops::DerefMut for Chunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.memory
    }
}
