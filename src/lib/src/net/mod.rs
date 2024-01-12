use crate::{crypto::Key, error::unexpected_message};
use anyhow::Result;
use chacha20poly1305::XNonce;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::Level;
use uuid::Uuid;

mod message;
pub use message::*;

pub mod types;

pub const MESSAGE_TIMEOUT: Option<Duration> = Some(Duration::from_secs(3));

pub const CHUNK_PARTS: usize = 2_000;
pub const CHUNK_PART_SIZE: usize = 512;
pub const CHUNK_SIZE: usize = CHUNK_PART_SIZE * CHUNK_PARTS;

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
        writer.write_all(nonce.as_slice()).await?;
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
        let mut nonce = XNonce::default();
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

pub async fn negotiate_hello<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    key: &Key,
) -> Result<()> {
    let stamp = Uuid::new_v4().into_bytes();

    send_message(
        &mut stream,
        key,
        Message::Hello(stamp),
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    match receive_message(&mut stream, key, MESSAGE_TIMEOUT).await? {
        Message::Hello(stamp) => {
            send_message(
                &mut stream,
                key,
                Message::Echo(stamp),
                MESSAGE_TIMEOUT,
                true,
            )
            .await
        }

        message => unexpected_message("Message::Hello", message),
    }?;

    match receive_message(&mut stream, key, MESSAGE_TIMEOUT).await? {
        Message::Echo(restamp) if restamp == stamp => Ok(()),

        Message::Echo(restamp) => {
            bail!("Peer reponded with the incorrect stamp: expected {stamp:?}, got {restamp:?}")
        }

        message => unexpected_message("Message::Echo", message),
    }
}

pub async fn ping_pong<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, key: &Key) -> Result<()> {
    send_message(&mut stream, key, Message::Ping, MESSAGE_TIMEOUT, true).await?;

    match receive_message(&mut stream, key, MESSAGE_TIMEOUT).await? {
        Message::Pong => Ok(()),
        message => unexpected_message("Message::Pong", message),
    }
}
