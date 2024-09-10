use std::time::Duration;
mod message;
pub use message::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("message error")]
    Message(#[from] message::Error),

    #[error("error using pooled buffers")]
    Pool(#[from] deadpool::unmanaged::PoolError),

    #[error("async IO error")]
    Io(#[from] tokio::io::Error),

    #[error("function timed out")]
    Timeout(#[from] tokio::time::error::Elapsed),

    // #[error("cryptographic error")]
    // Crypto(#[from] crate::crypto::Error),

    #[error("expected pong, received: {0:?}")]
    ExpectedPong(Message),
}

pub type Result<T> = std::result::Result<T, Error>;

pub const MESSAGE_TIMEOUT: Duration = Duration::from_secs(3);
pub const MAX_TIMEOUT: Duration = Duration::MAX;

pub struct Connection<S: AsyncRead + AsyncWrite + Unpin> {
    stream: S,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    pub fn new(cipher: XChaCha20Poly1305, stream: S) -> Self {
        Self {
            send_bufs: (Buf::new(), Buf::new()),
            recv_bufs: (Buf::new(), Buf::new()),
            cipher,
            stream,
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn send_message(
        &mut self,
        message: Message,
        timeout: Duration,
        flush: bool,
    ) -> Result<()> {
        let (serialize_buf, encrypt_buf) = &mut self.send_bufs;

        tokio::time::timeout(timeout, async {
            // Serialize message
            message.serialize_into(serialize_buf)?;

            // Encrypt message
            let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
            encrypt(&self.cipher, &nonce, serialize_buf.as_ref(), encrypt_buf)?;

            event!(Level::TRACE, ?nonce);

            // Transmit message
            let len = u32::try_from(encrypt_buf.len()).unwrap();
            self.stream.write_u32_le(len).await?;
            self.stream.write_all(&nonce).await?;
            self.stream.write_all(encrypt_buf.as_ref()).await?;

            if flush {
                self.stream.flush().await?;
            }

            Result::<()>::Ok(())
        })
        .await??;

        serialize_buf.clear();
        encrypt_buf.clear();

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn receive_message(&mut self, timeout: Duration) -> Result<Message> {
        let (deserialize_buf, decrypt_buf) = &mut self.recv_bufs;

        tokio::time::timeout(timeout, async {
            let len = usize::try_from(self.stream.read_u32_le().await?).unwrap();

            let mut nonce = XNonce::default();
            self.stream.read_exact(&mut nonce).await?;
            self.stream.read_exact(&mut decrypt_buf).await?;

            crate::crypto::decrypt(&self.cipher, &nonce, decrypt_buf, deserialize_buf)?;

            let message = Message::deserialize_from(&**deserialize_buf).await?;
            event!(Level::TRACE, ?message, ?len, ?nonce);

            self.msg_buf.clear();
            self.crypt_buf.clear();

            Ok(message)
        })
        .await?;

        deserialize_buf.clear();
        decrypt_buf.clear();

        Ok(())
    }

    pub async fn ping_pong(&mut self) -> Result<()> {
        self.send_message(Message::Ping, MESSAGE_TIMEOUT, true)
            .await?;

        match self.receive_message(MESSAGE_TIMEOUT).await? {
            Message::Pong => Ok(()),
            message => Err(Error::ExpectedPong(message)),
        }
    }
}