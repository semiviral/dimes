use serde::{Deserialize, Serialize};

pub mod chunk;

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub endpoint: String,
    pub max_chunks: u64,
}
