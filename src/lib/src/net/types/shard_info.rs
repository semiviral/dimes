use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    id: Uuid,
    agent: String,
    // TODO ensure it's less than i64::MAX
    max_chunks: i64,
}

impl ShardInfo {
    #[inline]
    pub fn new(id: Uuid, agent: String, max_chunks: u64) -> Result<Self> {
        let max_chunks = max_chunks.try_into()?;

        Ok(Self {
            id,
            agent,
            max_chunks,
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
