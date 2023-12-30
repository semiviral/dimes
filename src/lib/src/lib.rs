#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate tracing;

pub mod crypto;
pub mod message;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ShardConnectInfo {
    endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectInfo {
    pub agent: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShardInfo {
    pub max_chunks: u64,
}
