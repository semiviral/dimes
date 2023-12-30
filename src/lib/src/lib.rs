#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate tracing;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod api;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectInfo {
    pub agent: String,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct Token(Uuid);
