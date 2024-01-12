use anyhow::Result;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShardInfo {
    id: Uuid,
    max_chunks: i64,
}

impl ShardInfo {
    #[inline]
    pub fn new(id: Uuid, max_chunks: u64) -> Result<Self> {
        Ok(Self {
            id,
            max_chunks: max_chunks.try_into()?,
        })
    }

    #[inline]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    pub fn max_chunks(&self) -> i64 {
        self.max_chunks
    }
}
