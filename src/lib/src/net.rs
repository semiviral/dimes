use crate::{bstr::BStr, chunk::Chunk};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] tokio::io::Error),

    #[error("bincode error")]
    Coding(#[from] bincode::Error),

    #[error("agent string is not valid UTF-8")]
    MessageAgentInvalidUtf8(#[from] std::str::Utf8Error),

    #[error("invalid discriminant: {0:#X}")]
    MessageInvalidDiscriminant(u32),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Connection<IO: AsyncRead + AsyncWrite + Unpin> {
    stream: IO,
    recv_buf: Vec<u8>,
    send_buf: Vec<u8>,
}

impl<IO: AsyncRead + AsyncWrite + Unpin> Connection<IO> {
    pub fn new(stream: IO) -> Self {
        Self {
            stream,
            recv_buf: Vec::new(),
            send_buf: Vec::new(),
        }
    }

    pub async fn send(&mut self, message: Message, flush: bool) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        bincode::serialize_into(&mut self.send_buf, &message)?;

        let message_len = self.send_buf.len().try_into().unwrap();
        self.stream.write_u64_le(message_len).await?;
        self.stream.write_all(&self.send_buf).await?;

        if flush {
            self.stream.flush().await?;
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Message> {
        use tokio::io::AsyncReadExt;

        let message_len = self.stream.read_u64_le().await?;
        self.recv_buf.resize(message_len.try_into().unwrap(), 0);

        self.stream.read_exact(&mut self.recv_buf).await?;
        let message = bincode::deserialize(&self.recv_buf)?;

        Ok(message)
    }
}

#[repr(u32)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Ok = 0x0,

    Ping = 0x1,
    Pong = 0x2,

    AssignId { id: Uuid } = 0x3,

    ServerInfo { agent: BStr<64> } = 0x40,
    ShardInfo { chunks: u64, agent: BStr<64> } = 0x41,

    ShardStore { chunk: Chunk } = 0x100,
    ShardRetrieve { id: Uuid } = 0x101,
    ShardChunkExists { id: Uuid } = 0x102,
}
