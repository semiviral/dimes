use uuid::Uuid;

pub struct ShardInfo {
    pub id: Uuid,
    pub agent: String,
    pub max_chunks: i64,
}
