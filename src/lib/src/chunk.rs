use uuid::Uuid;

const CHUNK_SIZE: usize = 64_000;
pub type Chunk = [u8; CHUNK_SIZE];

#[derive(Debug)]
pub struct ChunkOwned {
    id: Uuid,
    memory: Box<Chunk>,
}

impl ChunkOwned {
    pub const SIZE: usize = CHUNK_SIZE;

    pub fn new_zeroed(id: Uuid) -> Self {
        use std::alloc::Layout;

        const MEMORY_LAYOUT: Layout = Layout::new::<Chunk>();

        // SAFETY: Allocate a zero-initialized `Chunk`-sized region of memory, and read it into box holding a `Chunk`.
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

impl std::ops::Deref for ChunkOwned {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl std::ops::DerefMut for ChunkOwned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.memory
    }
}
