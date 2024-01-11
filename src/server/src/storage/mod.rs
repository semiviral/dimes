use anyhow::Result;
use sqlx::PgPool;

pub mod types;

pub struct Storage {
    pool: PgPool,
}

impl Storage {
    pub async fn add_shard(&self, shard: types::ShardInfo) -> Result<()> {
        query!(
            "INSERT INTO shards VALUES ($1, $2, $3)",
            shard.id,
            shard.agent,
            shard.max_chunks
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
