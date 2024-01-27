use crate::{
    pools::{get_string_buf, ManagedString},
    Chunk, ChunkPart, Hash,
};
use anyhow::Result;
use std::mem::{size_of, Discriminant};
use uuid::Uuid;

#[repr(u32)]
#[derive(Debug)]
pub enum Message {
    Hello([u8; 16]) = Self::HELLO,
    Echo([u8; 16]) = Self::ECHO,

    Ok = Self::OK,

    Ping = Self::PING,
    Pong = Self::PONG,

    ShardInfo {
        id: Uuid,
        agent: ManagedString,
        chunks: u32,
    } = Self::SHARD_INFO,

    ShardShutdown = Self::SHARD_SHUTDOWN,

    Store {
        hash: Hash,
        chunk: Chunk,
    } = Self::STORE,

    StoreExists {
        hash: Hash,
    } = Self::STORE_EXISTS,

    Stock {
        hash: Hash,
        chunk: Chunk,
    } = Self::STOCK,

    RequestStock {
        hash: Hash,
    } = Self::REQUEST_STOCK,
}

#[allow(clippy::inconsistent_digit_grouping)]
impl Message {
    const OK: u32 = 0;
    const HELLO: u32 = 1_0_000;
    const ECHO: u32 = 1_0_001;
    const PING: u32 = 1_0_002;
    const PONG: u32 = 1_0_003;
    const SHARD_INFO: u32 = 2_0_000;
    const SHARD_SHUTDOWN: u32 = 2_1_000;
    const STORE: u32 = 3_0_000;
    const STOCK: u32 = 3_0_001;
    const STORE_EXISTS: u32 = 3_1_000;
    const REQUEST_STOCK: u32 = 3_0_010;

    pub async fn deserialize(bytes: &[u8]) -> Result<Self> {
        let (discriminant, raw) = bytes.split_at(std::mem::size_of::<Discriminant<Message>>());
        let discriminant = u32::from_le_bytes(discriminant.try_into().unwrap());

        match discriminant {
            Self::OK => Ok(Self::Ok),

            Self::HELLO => Ok(Self::Hello(raw.try_into().expect("wrong data length"))),
            Self::ECHO => Ok(Self::Echo(raw.try_into().expect("wrong data length"))),

            Self::PING => Ok(Self::Ping),
            Self::PONG => Ok(Self::Pong),

            Self::SHARD_INFO => {
                let (id_bytes, raw) = raw.split_at(size_of::<Uuid>());
                let id = Uuid::from_bytes_le(id_bytes.try_into().unwrap());

                let (agent_str_len_bytes, raw) = raw.split_at(size_of::<u32>());
                let agent_str_len = u64::from_le_bytes(agent_str_len_bytes.try_into().unwrap());

                let (agent_str_bytes, chunks_bytes) = raw.split_at(agent_str_len as usize);
                let agent_str =
                    std::str::from_utf8(agent_str_bytes).expect("received invalid agent string");
                let mut agent = get_string_buf().await;
                agent.push_str(agent_str);

                let chunks = u32::from_le_bytes(chunks_bytes.try_into().unwrap());

                Ok(Self::ShardInfo { id, agent, chunks })
            }

            discriminant => bail!("Unknown discriminant: {discriminant}"),
        }
    }

    pub fn serialize(self, buf: &mut Vec<u8>) -> Result<()> {
        debug_assert!(buf.is_empty());

        // SAFETY: The discriminant value is the first value in the struct's memory representation.
        let discriminant = unsafe { (&self as *const Self as *const u32).read() };
        buf.extend_from_slice(&discriminant.to_le_bytes());

        match self {
            Self::Ok | Self::Ping | Self::Pong | Self::ShardShutdown => {
                // do nothing
            }

            Self::Hello(stamp) | Self::Echo(stamp) => {
                buf.extend_from_slice(&stamp);
            }

            Self::ShardInfo { id, agent, chunks } => {
                buf.extend_from_slice(&id.to_bytes_le());
                buf.extend_from_slice(&(agent.len() as u64).to_le_bytes());
                buf.extend_from_slice(agent.as_bytes());
                buf.extend_from_slice(&chunks.to_le_bytes());
            }

            Self::Store { hash, chunk } | Self::Stock { hash, chunk } => {
                buf.extend_from_slice(&hash.into_bytes());
                buf.extend_from_slice(&(chunk.len() as u64).to_le_bytes());
                buf.extend_from_slice(chunk.as_slice());
            }

            Self::StoreExists { hash } | Self::RequestStock { hash } => {
                buf.extend_from_slice(&hash.into_bytes());
            }
        }

        Ok(())
    }
}
