use anyhow::Result;
use lib::ShardInfo;
use sqlx::PgPool;

#[derive(Debug)]
pub struct DbStore {
    pool: PgPool,
}

impl DbStore {
    #[inline]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_shard(&self, shard: &ShardInfo) -> Result<()> {
        // query!(
        //     "INSERT INTO shards VALUES ($1, $2, $3, 0)",
        //     shard.id(),
        //     shard.agent(),
        //     shard.chunks()
        // )
        // .execute(&self.pool)
        // .await?;

        Ok(())
    }
}
