use crate::{
    array_pool::{ArrayPool, ManagedArray},
    LIB_RUNTIME,
};
use once_cell::sync::Lazy;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use uuid::Uuid;

const CHUNK_SIZE: usize = 64_000;

static CHUNK_ARRAYS: Lazy<ArrayPool<CHUNK_SIZE>> = Lazy::new(|| ArrayPool::new(512));

async fn get_chunk_array() -> ManagedArray<CHUNK_SIZE> {
    CHUNK_ARRAYS
        .get()
        .await
        .expect("array pool did not return new array")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    id: Uuid,

    #[serde(deserialize_with = "deserialize_chunk_array")]
    memory: ManagedArray<CHUNK_SIZE>,
}

impl Chunk {
    pub const SIZE: usize = CHUNK_SIZE;

    pub async fn new_zeroed(id: Uuid) -> Self {
        Self {
            id,
            memory: CHUNK_ARRAYS
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

fn deserialize_chunk_array<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<ManagedArray<CHUNK_SIZE>, D::Error> {
    deserializer.deserialize_bytes(ChunkArrayVisitor)
}

struct ChunkArrayVisitor;

impl Visitor<'_> for ChunkArrayVisitor {
    type Value = ManagedArray<CHUNK_SIZE>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a byte array w/ len {}", CHUNK_SIZE)
    }

    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        if v.len() != CHUNK_SIZE {
            Err(E::invalid_length(v.len(), &self))
        } else {
            let mut chunk_array = LIB_RUNTIME.block_on(get_chunk_array());
            chunk_array.copy_from_slice(v);

            Ok(chunk_array)
        }
    }
}
