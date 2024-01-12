use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct DbStore {
    pool: PgPool,
}

impl DbStore {
    #[inline]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_shard(&self, agent: &str, id: Uuid, max_chunks: i64) -> Result<()> {
        query!(
            "INSERT INTO shards VALUES ($1, $2, $3, 0)",
            id,
            agent,
            max_chunks
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
