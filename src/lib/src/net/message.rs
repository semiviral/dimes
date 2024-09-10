use crate::{bstr::BStr, chunk::Chunk, net::Error};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[repr(u32)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Ok = Self::OK,

    Ping = Self::PING,

    Pong = Self::PONG,

    AssignId { id: Uuid } = Self::ASSIGN_ID,

    ServerInfo { agent: BStr<64> },
    ShardInfo { chunks: u64, agent: BStr<64> } = Self::SHARD_INFO,

    ShardStore { chunk: Chunk } = Self::SHARD_STORE,
    ShardRetrieve { id: Uuid } = Self::SHARD_RETRIEVE,
    ShardChunkExists { id: Uuid } = Self::SHARD_CHUNK_EXISTS,
}

impl Message {
    // GENERAL
    const OK: u32 = 0x0;
    const PING: u32 = 0x1;
    const PONG: u32 = 0x2;
    const ASSIGN_ID: u32 = 0x3;
    const REQUEST_INFO: u32 = 0x4;
    // TO SERVER
    const SHARD_INFO: u32 = 0x80000;
    // TO SHARD
    const SHARD_STORE: u32 = 0x80100;
    const SHARD_RETRIEVE: u32 = 0x80101;
    const SHARD_CHUNK_EXISTS: u32 = 0x80102;

    pub async fn send(self, mut stream: impl AsyncWrite + Unpin) -> Result<()> {
        async fn write_uuid(mut stream: impl AsyncWrite + Unpin, uuid: Uuid) -> Result<()> {
            let uuid_bytes = uuid.to_bytes_le();
            stream.write_all(&uuid_bytes).await?;

            Ok(())
        }

        // Safety: https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        let discriminant = unsafe { (&self as *const _ as *const u32).read() };
        stream.write_u32_le(discriminant).await?;

        match self {
            Self::Ok | Self::Ping | Self::Pong => {}

            Self::AssignId { id } => {
                write_uuid(&mut stream, id).await?;
            }

            Self::ServerInfo { agent } => {
                stream.write_all(agent.as_bytes()).await?;
            }

            Self::ShardInfo { agent, chunks } => {
                stream.write_u64_le(chunks).await?;
                stream.write_all(agent.as_bytes()).await?;
            }

            Self::ShardStore { chunk } => {
                write_uuid(&mut stream, chunk.id()).await?;
                stream.write_all(chunk.as_slice()).await?;
            }

            Self::ShardRetrieve { id } | Self::ShardChunkExists { id } => {
                write_uuid(&mut stream, id).await?;
            }
        }

        Ok(())
    }

    pub async fn recv(mut stream: impl AsyncRead + Unpin) -> Result<Self> {
        bincode::deserialize_from(reader)
        async fn read_uuid(mut stream: impl AsyncRead + Unpin) -> Result<Uuid> {
            let mut buf = [0u8; size_of::<Uuid>()];
            stream.read_exact(&mut buf).await?;
            let uuid = Uuid::from_bytes_le(buf);

            Ok(uuid)
        }

        let discriminant = stream.read_u32_le().await?;

        match discriminant {
            Self::OK => Ok(Self::Ok),

            Self::PING => Ok(Self::Ping),
            Self::PONG => Ok(Self::Pong),

            Self::SHARD_INFO => {
                let id = read_uuid(&mut stream).await?;
                let chunks = stream.read_u64_le().await?;

                // TODO: `read_string` or something
                let mut agent_buf = [0u8; u8::MAX as usize];
                let agent_len = usize::from(stream.read_u8().await?);
                let agent_buf = &mut agent_buf[..agent_len];

                stream.read_exact(agent_buf).await?;
                let agent = std::str::from_utf8(agent_buf)?.to_owned();

                Ok(Self::ShardInfo { chunks, agent })
            }

            Self::SHARD_STORE => {
                let id = read_uuid(&mut stream).await?;
                let mut chunk = Chunk::new_zeroed(id).await;

                stream.read_exact(chunk.as_mut_slice()).await?;

                Ok(Self::ShardStore { chunk })
            }

            Self::SHARD_RETRIEVE => Ok(Self::ShardRetrieve {
                id: read_uuid(&mut stream).await?,
            }),

            Self::SHARD_CHUNK_EXISTS => Ok(Self::ShardChunkExists {
                id: read_uuid(&mut stream).await?,
            }),

            discriminant => Err(Error::MessageInvalidDiscriminant(discriminant)),
        }
    }
}
