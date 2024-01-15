use crate::net::types::{ChunkHash, ChunkPart};
use anyhow::Result;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

#[repr(u32)]
#[derive(Debug)]
pub enum Message {
    Hello([u8; 16]) = 0,
    Echo([u8; 16]) = 1,

    Ok = 2,

    Ping = 20,
    Pong = 21,

    ShardInfo {
        id: Uuid,
        agent: String,
        chunks: u64,
    } = 2000,

    ShardShutdown = 1000,

    PrepareStore {
        hash: ChunkHash,
    } = 6000,

    AlreadyStoring {
        hash: ChunkHash,
    } = 5000,

    PrepareStock {
        hash: ChunkHash,
    } = 6001,

    ChunkPart {
        hash: ChunkHash,
        part: ChunkPart,
    } = 6002,
}

impl Message {
    pub async fn serialize_to(self, mut stream: impl AsyncWrite + Unpin) -> Result<()> {
        // SAFETY: The discriminant value is the first value in the struct's memory representation.
        let discriminant = unsafe { (&self as *const Self as *const u32).read() };
        stream.write_u32_le(discriminant).await?;

        match self {
            Self::Hello(stamp) | Self::Echo(stamp) => {
                stream.write_all(&stamp).await?;
            }

            Self::Ok | Self::Ping | Self::Pong | Self::ShardShutdown => {
                // do nothing
            }

            Self::ShardInfo { id, agent, chunks } => {
                stream.write_all(&id.to_bytes_le()).await?;
                stream.write_all(agent.as_bytes()).await?;
                stream.write_all(&chunks.to_be_bytes()).await?;
            }

            Self::PrepareStore { hash }
            | Self::AlreadyStoring { hash }
            | Self::PrepareStock { hash } => {
                stream.write_all(&hash.into_bytes()).await?;
            }

            Self::ChunkPart { hash, part } => {
                stream.write_all(&hash.into_bytes()).await?;
                stream.write_all(&*part).await?;
            }
        }

        Ok(())
    }
}
