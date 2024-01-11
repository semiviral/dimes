use crate::net::types::ShardInfo;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use uuid::Uuid;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Hello([u8; 16]),
    Echo([u8; 16]),

    Ping,
    Pong,

    Info(ShardInfo),

    PrepareStore {
        id: Uuid,
    },

    PrepareStock {
        id: Uuid,
    },

    #[serde(with = "BigArray")]
    ChunkPart([u8; super::CHUNK_PART_SIZE]),
}
