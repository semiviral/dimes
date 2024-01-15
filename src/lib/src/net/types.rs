use anyhow::Result;
use uuid::Uuid;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkHash([u8; 16]);

impl ChunkHash {
    #[inline]
    pub const fn into_bytes(self) -> [u8; 16] {
        self.0
    }
}

pub type ChunkPart = Box<[u8; super::CHUNK_PART_SIZE]>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShardInfo {
    id: Uuid,
    agent: String,
    max_chunks: i64,
}

impl ShardInfo {
    #[inline]
    pub fn new(id: Uuid, agent: String, max_chunks: usize) -> Result<Self> {
        Ok(Self {
            id,
            agent,
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
