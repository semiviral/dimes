use blake2::digest::{Update, VariableOutput};
use chacha20poly1305::{aead::Buffer, AeadInPlace, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use tokio::io::{AsyncRead, AsyncWrite};
use x25519_dalek::{EphemeralSecret, PublicKey};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ECDH handshake write error.")]
    HandshakeWrite,
    #[error("ECDH handshake flush error.")]
    HandshakeFlush,
    #[error("ECDH handshake read error.")]
    HandshakeRead,
    #[error("ECDH handshake finalize error.")]
    HandshakeFinalize,

    #[error("Opaque cipher error.")]
    Cipher,
}

pub type Result<T> = std::result::Result<T, Error>;

const XCHACHA20_POLY1305_KEY_SIZE: usize = 32;
pub type Key = [u8; XCHACHA20_POLY1305_KEY_SIZE];

pub async fn ecdh_handshake<R: AsyncRead + AsyncWrite + Unpin>(mut stream: R) -> Result<Key> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let private_key = EphemeralSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&private_key);

    // Send public key to peer.
    stream
        .write_all(public_key.as_bytes())
        .await
        .map_err(|_| Error::HandshakeWrite)?;
    stream.flush().await.map_err(|_| Error::HandshakeFlush)?;

    // Get public key from peer.
    let mut peer_public_key = [0u8; size_of::<PublicKey>()];
    stream
        .read_exact(&mut peer_public_key)
        .await
        .map_err(|_| Error::HandshakeRead)?;
    let peer_public_key = PublicKey::from(peer_public_key);

    // Calculate shared secret.
    let dh_secret = private_key.diffie_hellman(&peer_public_key);
    let mut kdf = blake2::Blake2bVar::new(XCHACHA20_POLY1305_KEY_SIZE).unwrap();
    kdf.update(dh_secret.as_bytes());

    // Generate encryption key from shared secret.
    let mut key = [0u8; XCHACHA20_POLY1305_KEY_SIZE];
    kdf.finalize_variable(&mut key)
        .map_err(|_| Error::HandshakeFinalize)?;

    Ok(key)
}

pub fn encrypt(
    cipher: &XChaCha20Poly1305,
    nonce: &XNonce,
    data: impl AsRef<[u8]>,
    buffer: &mut dyn Buffer,
) -> Result<()> {
    cipher
        .encrypt_in_place(nonce, data.as_ref(), buffer)
        .map_err(|_| Error::Cipher)?;

    Ok(())
}

pub fn decrypt(
    cipher: &XChaCha20Poly1305,
    nonce: &XNonce,
    data: impl AsRef<[u8]>,
    buffer: &mut dyn Buffer,
) -> Result<()> {
    cipher
        .decrypt_in_place(nonce, data.as_ref(), buffer)
        .map_err(|_| Error::Cipher)?;

    Ok(())
}
