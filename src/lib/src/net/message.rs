use std::mem::Discriminant;

use crate::chunk::Chunk;
use deadpool::unmanaged::{Object, Pool};
use once_cell::sync::Lazy;
use tokio::io::ReadBuf;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[repr(u32)]
#[derive(Debug)]
pub enum Message {
    Ok = 0x0,

    Hello {
        stamp: Uuid,
    } = 0x1,

    Echo {
        stamp: Uuid,
    } = 0x2,

    Ping {
        stamp: Uuid,
    } = 0x4,

    Pong {
        stamp: Uuid,
    } = 0x5,

    ShardInfo {
        id: Uuid,
        agent: String,
        chunks: u32,
    } = 0x80000,

    Store {
        id: Uuid,
        data: Box<Chunk>,
    } = 0x80100,

    Retrieve {
        id: Uuid,
    } = 0x80101,

    ChunkExists {
        id: Uuid,
    } = 0x80102,
}

impl Message {
    pub async fn serialize_into(self, buf: &mut Vec<u8>) -> Result<()> {
        // Safety: https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        let discriminant = unsafe { (&self as *const _ as *const u32).read() };
        buf.extend(discriminant.to_le_bytes());

        match self {
            Message::Ok => {}

            Message::Hello { stamp }
            | Message::Echo { stamp }
            | Message::Ping { stamp }
            | Message::Pong { stamp } => {
                buf.extend(stamp.to_bytes_le());
            }

            Message::ShardInfo { id, agent, chunks } => {
                buf.extend(id.to_bytes_le());
                buf.extend(agent.len().to_le_bytes());
                buf.extend(agent.as_bytes());
                buf.extend(chunks.to_le_bytes());
            }

            Message::Store { id, data } => {
                buf.extend(id.to_bytes_le());
                buf.extend(data.as_slice());
            }

            Message::Retrieve { id } | Message::ChunkExists { id } => {
                buf.extend(id.to_bytes_le());
            }
        }

        Ok(())
    }

    pub fn deserialize_from(buf: &[u8]) -> Result<Self> {
        let discriminant = u32::from_le_bytes(buf[..size_of::<u32>()].try_into().unwrap());
        match discriminant {}
    }
}
