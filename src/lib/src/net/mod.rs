use crate::{error::unexpected_message, pools::get_message_buf};
use anyhow::Result;
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::Level;
use uuid::Uuid;

mod message;
pub use message::*;

pub const MESSAGE_TIMEOUT: Duration = Duration::from_secs(3);
pub const MAX_TIMEOUT: Duration = Duration::MAX;

#[instrument(level = "trace", skip(writer, cipher))]
pub async fn send_message<W: AsyncWrite + Unpin>(
    mut writer: W,
    cipher: &XChaCha20Poly1305,
    message: Message,
    timeout: Duration,
    flush: bool,
) -> Result<()> {
    tokio::time::timeout(timeout, async {
        let mut message_buf = get_message_buf().await;
        let mut encryption_buf = get_message_buf().await;

        message
            .serialize(&mut message_buf)
            .expect("failed to serialize message");

        let nonce = crate::crypto::encrypt(cipher, &message_buf, &mut *encryption_buf)?;

        event!(Level::TRACE, ?nonce);

        writer.write_u32_le(encryption_buf.len() as u32).await?;
        writer.write_all(nonce.as_slice()).await?;
        writer.write_all(&encryption_buf).await?;

        if flush {
            writer.flush().await?;
        }

        Result::<()>::Ok(())
    })
    .await??;

    Ok(())
}

#[instrument(level = "trace", skip(reader, cipher))]
pub async fn receive_message<R: AsyncRead + Unpin, C: AsRef<XChaCha20Poly1305>>(
    mut reader: R,
    cipher: C,
    timeout: Duration,
) -> Result<Message> {
    tokio::time::timeout(timeout, async {
        let mut message_buf = get_message_buf().await;
        let mut decryption_buf = get_message_buf().await;

        let data_len = reader.read_u32_le().await? as usize;
        let mut nonce = XNonce::default();
        reader.read_exact(&mut nonce).await?;
        reader.read_exact(&mut decryption_buf).await?;

        crate::crypto::decrypt(cipher.as_ref(), &nonce, &decryption_buf, &mut *message_buf)?;

        let message = Message::deserialize(&message_buf).await?;
        event!(Level::TRACE, ?message, ?data_len, ?nonce);

        Ok(message)
    })
    .await?
}

pub async fn negotiate_hello<S: AsyncRead + AsyncWrite + Unpin, C: AsRef<XChaCha20Poly1305>>(
    mut stream: S,
    cipher: C,
) -> Result<()> {
    let stamp = Uuid::new_v4().into_bytes();

    send_message(
        &mut stream,
        cipher.as_ref(),
        Message::Hello(stamp),
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    match receive_message(&mut stream, &cipher, MESSAGE_TIMEOUT).await? {
        Message::Hello(stamp) => {
            send_message(
                &mut stream,
                cipher.as_ref(),
                Message::Echo(stamp),
                MESSAGE_TIMEOUT,
                true,
            )
            .await
        }

        message => unexpected_message("Message::Hello", message),
    }?;

    match receive_message(&mut stream, &cipher, MESSAGE_TIMEOUT).await? {
        Message::Echo(restamp) if restamp == stamp => Ok(()),

        Message::Echo(restamp) => {
            bail!("Peer reponded with the incorrect stamp: expected {stamp:?}, got {restamp:?}")
        }

        message => unexpected_message("Message::Echo", message),
    }
}

pub async fn ping_pong<S: AsyncRead + AsyncWrite + Unpin, C: AsRef<XChaCha20Poly1305>>(
    mut stream: S,
    cipher: C,
) -> Result<()> {
    send_message(
        &mut stream,
        cipher.as_ref(),
        Message::Ping,
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    match receive_message(&mut stream, &cipher, MESSAGE_TIMEOUT).await? {
        Message::Pong => Ok(()),
        message => unexpected_message("Message::Pong", message),
    }
}
