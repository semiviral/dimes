#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate tracing;

use serde::{Deserialize, Serialize};

pub mod api;
pub mod error_code;
pub mod token;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectInfo<K: token::Kind> {
    pub agent: String,
    pub token: token::Token<K>,
}

pub type ChunkHash = [u8; 64];
