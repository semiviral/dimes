use anyhow::Result;
use sqlx::PgPool;

pub mod types;

pub struct Storage {
    pool: PgPool,
}

impl Storage {
    pub async fn add_shard(&self, shard: types::ShardInfo) -> Result<()> {
        query_as!(types::ShardInfo, "INSERT INTO shards VALUES (?, ?, ?)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
