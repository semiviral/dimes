use uuid::Uuid;

pub struct ShardInfo {
    id: Uuid,
    agent: String,
    max_chunks: u32,
}
