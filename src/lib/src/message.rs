use crate::crypto::{Key, Nonce};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Ping { stamp: [u8; 16] },
    Pong { restamp: [u8; 16] },
}

pub async fn send_message<W: AsyncWrite + Unpin>(
    mut writer: W,
    key: &Key,
    message: Message,
) -> Result<()> {
    let message_bytes = bincode::serialize(&message)?;
    let (nonce, encrypted_data) = crate::crypto::encrypt(key, &message_bytes)?;

    writer.write_u32_le(encrypted_data.len() as u32).await?;
    writer.write_all(&nonce).await?;
    writer.write_all(&encrypted_data).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn receive_message<R: AsyncRead + Unpin>(mut reader: R, key: &Key) -> Result<Message> {
    let len = reader.read_u32_le().await? as usize;
    let mut nonce = Nonce::default();
    reader.read_exact(&mut nonce).await?;

    trace!("Receiving message: {len} / {nonce:X?}");

    let mut data = vec![0u8; len];
    let read_len = reader.read_exact(&mut data).await?;
    assert_eq!(read_len, len);

    let decrypted_bytes = crate::crypto::decrypt(key, &nonce, &data)?;
    let message = bincode::deserialize(&decrypted_bytes)?;

    Ok(message)
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
