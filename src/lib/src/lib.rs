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
pub struct ChunkHash([u8; 16]);

impl ChunkHash {
    #[inline]
    pub const fn into_bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShardInfo<'a> {
    id: Uuid,
    agent: &'a str,
    max_chunks: i64,
}

impl<'a> ShardInfo<'a> {
    #[inline]
    pub fn new(id: Uuid, agent: impl AsRef<str> + 'a, max_chunks: usize) -> anyhow::Result<Self> {
        Ok(Self {
            id,
            agent: agent.as_ref(),
            max_chunks: max_chunks.try_into()?,
        })
    }

    #[inline]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    pub fn agent(&self) -> &str {
        self.agent.as_ref()
    }

    #[inline]
    pub fn max_chunks(&self) -> i64 {
        self.max_chunks
    }
}
