use pools::ManagedString;
use uuid::Uuid;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate tracing;

pub mod crypto;
pub mod error;
pub mod net;
pub mod pools;

pub const CHUNK_PARTS: usize = 0x100; // 256
pub const CHUNK_PART_SIZE: usize = 0x1000; // 4096
pub const CHUNK_SIZE: usize = CHUNK_PART_SIZE * CHUNK_PARTS; // 1MiB

pub type Chunk = Box<[u8; CHUNK_SIZE]>;
pub type ChunkPart = Box<[u8; CHUNK_PART_SIZE]>;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash([u8; 16]);

impl Hash {
    #[inline]
    pub const fn into_bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShardInfo {
    id: Uuid,
    agent: ManagedString,
    chunks: i64,
}

impl ShardInfo {
    #[inline]
    pub fn new(id: Uuid, agent: ManagedString, chunks: u32) -> Self {
        Self {
            id,
            agent,
            chunks: chunks.into(),
        }
    }

    #[inline]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    pub fn agent(&self) -> &str {
        self.agent.as_str()
    }

    #[inline]
    pub fn chunks(&self) -> i64 {
        self.chunks
    }
}
