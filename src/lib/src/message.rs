use std::time::Duration;

use crate::crypto::{Key, Nonce};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::timeout,
};
use tracing::Level;
use uuid::Uuid;

pub const MESSAGE_TIMEOUT: Duration = Duration::from_secs(10);
pub const CHUNK_PARTS: usize = 2_000;
pub const CHUNK_PART_SIZE: usize = 256;
pub const CHUNK_SIZE: usize = CHUNK_PART_SIZE * CHUNK_PARTS;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Ping {
        stamp: [u8; 16],
    },
    Pong {
        restamp: [u8; 16],
    },

    Info {
        agent: String,
        version: String,
        max_chunks: u64,
    },

    PrepareStore {
        id: Uuid
    },

    PrepareReceive {
        id: Uuid,
    },

    #[serde(with = "BigArray")]
    ChunkPart([u8; 256]),
}

#[instrument(level = "trace", skip(writer, key, message))]
pub async fn send_message<W: AsyncWrite + Unpin>(
    writer: W,
    key: &Key,
    message: Message,
) -> Result<()> {
    async fn send_message_inner<W: AsyncWrite + Unpin>(
        mut writer: W,
        key: &Key,
        message: Message,
    ) -> Result<()> {
        let message_bytes = bincode::serialize(&message)?;
        let (nonce, encrypted_data) = crate::crypto::encrypt(key, &message_bytes)?;

        writer.write_u32_le(encrypted_data.len() as u32).await?;
        writer.write_all(&nonce).await?;
        writer.write_all(&encrypted_data).await?;

        event!(Level::TRACE, raw = ?message, crypted_len = %encrypted_data.len(), nonce = %format!("{nonce:X?}"));

        writer.flush().await?;

        Ok(())
    }

    timeout(MESSAGE_TIMEOUT, send_message_inner(writer, key, message)).await?
}

#[instrument(level = "trace", skip(writer, key, messages))]
pub async fn send_messages<W: AsyncWrite + Unpin>(
    writer: W,
    key: &Key,
    messages: impl Iterator<Item = Message>,
) -> Result<()> {
    async fn send_messages_inner<W: AsyncWrite + Unpin>(
        mut writer: W,
        key: &Key,
        messages: impl Iterator<Item = Message>,
    ) -> Result<()> {
        for message in messages {
            let message_bytes = bincode::serialize(&message)?;
            let (nonce, encrypted_data) = crate::crypto::encrypt(key, &message_bytes)?;

            writer.write_u32_le(encrypted_data.len() as u32).await?;
            writer.write_all(&nonce).await?;
            writer.write_all(&encrypted_data).await?;

            event!(Level::TRACE, raw = ?message, crypted_len = %encrypted_data.len(), nonce = %format!("{nonce:X?}"));
        }

        writer.flush().await?;

        Ok(())
    }

    timeout(MESSAGE_TIMEOUT, send_messages_inner(writer, key, messages)).await?
}

#[instrument(level = "trace", skip(reader, key))]
pub async fn receive_message<R: AsyncRead + Unpin>(reader: R, key: &Key) -> Result<Message> {
    pub async fn receive_message_inner<R: AsyncRead + Unpin>(
        mut reader: R,
        key: &Key,
    ) -> Result<Message> {
        let len = reader.read_u32_le().await? as usize;
        let mut nonce = Nonce::default();
        reader.read_exact(&mut nonce).await?;

        let mut data = vec![0u8; len];
        let read_len = reader.read_exact(&mut data).await?;
        assert_eq!(read_len, len);

        let decrypted_bytes = crate::crypto::decrypt(key, &nonce, &data)?;
        let message = bincode::deserialize(&decrypted_bytes)?;

        event!(Level::TRACE, raw = ?message, crypted_len = %decrypted_bytes.len(), nonce = %format!("{nonce:X?}"));

        Ok(message)
    }

    timeout(MESSAGE_TIMEOUT, receive_message_inner(reader, key)).await?
}

pub async fn ping_peer<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, key: &Key) -> Result<()> {
    let stamp = rand::random();
    send_message(&mut stream, key, Message::Ping { stamp }).await?;
    let Message::Pong { restamp } = receive_message(&mut stream, key).await? else {
        bail!("Peer responded with incorrect message type (expected Pong).");
    };

    if restamp == stamp {
        Ok(())
    } else {
        bail!("Peer failed to restamp the ping correctly.")
    }
}

pub async fn receive_chunk<R: AsyncRead + Unpin>(mut reader: R, key: &Key) -> Result<Box<[u8]>> {
    // TODO define a fixed chunk size to make this cacheable
    let mut chunk = vec![0u8; CHUNK_SIZE].into_boxed_slice();

    for empty_part in chunk.chunks_mut(CHUNK_PART_SIZE) {
        assert!(empty_part.len() == CHUNK_PART_SIZE);

        match receive_message(&mut reader, key).await? {
            Message::ChunkPart(part) => {
                empty_part.copy_from_slice(&part);
            }

            message => {
                bail!("Expected chunk part, got: {message:?}")
            }
        }
    }

    Ok(chunk)
}

pub async fn send_chunk<W: AsyncWrite + Unpin>(
    mut writer: W,
    key: &Key,
    id: Uuid,
    chunk: &[u8],
) -> Result<()> {
    send_message(&mut writer, key, Message::PrepareStore { id }).await?;
    send_messages(&mut writer, key, chunk.chunks(CHUNK_PART_SIZE).map(|part| Message::ChunkPart(part.try_into().unwrap()))).await?;

    Ok(())
}
