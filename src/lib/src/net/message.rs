use crate::net::types::{ChunkHash, ShardInfo};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Hello([u8; 16]),
    Echo([u8; 16]),

    Ping,
    Pong,

    Ok,

    ShardInfo(ShardInfo),
    ShardShutdown,

    PrepareStore {
        hash: ChunkHash,
    },
    AlreadyStoring {
        hash: ChunkHash,
    },

    PrepareStock {
        hash: ChunkHash,
    },

    ChunkPart {
        hash: ChunkHash,

        #[serde(with = "BigArray")]
        part: [u8; super::CHUNK_PART_SIZE],
    },
}
