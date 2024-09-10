use crate::array_pool::{ArrayPool, ManagedArray};
use once_cell::sync::Lazy;
use uuid::Uuid;

const CHUNK_SIZE: usize = 64_000;

static ARRAY_POOL: Lazy<ArrayPool<CHUNK_SIZE>> = Lazy::new(|| ArrayPool::new(512));

#[derive(Debug)]
pub struct Chunk {
    id: Uuid,
    memory: ManagedArray<CHUNK_SIZE>,
}

impl Chunk {
    pub const SIZE: usize = CHUNK_SIZE;

    pub async fn new_zeroed(id: Uuid) -> Self {
        Self {
            id,
            memory: ARRAY_POOL
                .get()
                .await
                .expect("array pool did not return new array"),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl std::ops::Deref for Chunk {
    type Target = Box<[u8; CHUNK_SIZE]>;

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl std::ops::DerefMut for Chunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.memory
    }
}
