use std::time::Duration;

use crate::crypto::{Key, Nonce};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::Level;
use uuid::Uuid;

pub const MESSAGE_TIMEOUT: Option<Duration> = Some(Duration::from_secs(3));

pub const CHUNK_PARTS: usize = 2_000;
pub const CHUNK_PART_SIZE: usize = 256;
pub const CHUNK_SIZE: usize = CHUNK_PART_SIZE * CHUNK_PARTS;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    // TODO remove stamps from pings, use a Hello for initial auth
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
        id: Uuid,
    },

    PrepareStock {
        id: Uuid,
    },

    #[serde(with = "BigArray")]
    ChunkPart([u8; 512]),
}

#[instrument(level = "trace", skip(writer, key, message))]
pub async fn send_message<W: AsyncWrite + Unpin>(
    writer: W,
    key: &Key,
    message: Message,
    timeout: Option<Duration>,
    flush: bool,
) -> Result<()> {
    async fn send_message_inner<W: AsyncWrite + Unpin>(
        mut writer: W,
        key: &Key,
        message: Message,
        flush: bool,
    ) -> Result<()> {
        let message_bytes = bincode::serialize(&message)?;
        let (nonce, encrypted_data) = crate::crypto::encrypt(key, &message_bytes)?;

        writer.write_u32_le(encrypted_data.len() as u32).await?;
        writer.write_all(&nonce).await?;
        writer.write_all(&encrypted_data).await?;

        event!(Level::TRACE,
            raw = ?message,
            crypted_len = %encrypted_data.len(),
            nonce = %format!("{nonce:X?}")
        );

        if flush {
            writer.flush().await?;
        }

        Ok(())
    }

    let send_message = send_message_inner(writer, key, message, flush);
    match timeout {
        Some(timeout) => tokio::time::timeout(timeout, send_message).await?,

        None => send_message.await,
    }
}

#[instrument(level = "trace", skip(reader, key))]
pub async fn receive_message<R: AsyncRead + Unpin>(
    reader: R,
    key: &Key,
    timeout: Option<Duration>,
) -> Result<Message> {
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

    let receive_message = receive_message_inner(reader, key);
    match timeout {
        Some(timeout) => tokio::time::timeout(timeout, receive_message).await?,

        None => receive_message.await,
    }
}

pub async fn ping_pong<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, key: &Key) -> Result<()> {
    let stamp = rand::random();
    send_message(
        &mut stream,
        key,
        Message::Ping { stamp },
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    let Message::Pong { restamp } = receive_message(&mut stream, key, MESSAGE_TIMEOUT).await?
    else {
        bail!("Peer responded with incorrect message type (expected Pong).");
    };

    if restamp == stamp {
        Ok(())
    } else {
        bail!("peer failed to restamp the ping correctly.")
    }
}

pub async fn receive_chunk<R: AsyncRead + AsyncWrite + Unpin>(
    mut reader: R,
    key: &Key,
) -> Result<Box<[u8]>> {
    let mut chunk = vec![0u8; CHUNK_SIZE].into_boxed_slice();

    for empty_part in chunk.chunks_mut(CHUNK_PART_SIZE) {
        assert!(empty_part.len() == CHUNK_PART_SIZE);

        match receive_message(&mut reader, key, MESSAGE_TIMEOUT).await? {
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
    chunk: &[u8],
) -> Result<()> {
    assert_eq!(chunk.len(), CHUNK_SIZE, "chunk is not the correct size");

    for part in chunk.chunks(CHUNK_PART_SIZE) {
        send_message(
            &mut writer,
            key,
            Message::ChunkPart(part.try_into().unwrap()),
            MESSAGE_TIMEOUT,
            false,
        )
        .await?;
    }

    writer.flush().await?;

    Ok(())
}
