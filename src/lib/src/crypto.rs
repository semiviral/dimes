use anyhow::Result;
use blake2::digest::{Update, VariableOutput};
use chacha20poly1305::{aead::Buffer, AeadCore, AeadInPlace, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use tokio::io::{AsyncRead, AsyncWrite};
use x25519_dalek::{EphemeralSecret, PublicKey};

const XCHACHA20_POLY1305_KEY_SIZE: usize = 32;

pub type Key = [u8; XCHACHA20_POLY1305_KEY_SIZE];

pub async fn ecdh_handshake<R: AsyncRead + AsyncWrite + Unpin>(mut stream: R) -> Result<Key> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let private_key = EphemeralSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&private_key);

    stream.write_all(public_key.as_bytes()).await?;
    stream.flush().await?;

    let mut peer_public_key = [0u8; core::mem::size_of::<PublicKey>()];
    stream.read_exact(&mut peer_public_key).await?;
    let peer_public_key = PublicKey::from(peer_public_key);

    let dh_secret = private_key.diffie_hellman(&peer_public_key);
    let mut kdf = blake2::Blake2bVar::new(XCHACHA20_POLY1305_KEY_SIZE).unwrap();
    kdf.update(dh_secret.as_bytes());

    let mut key = [0u8; XCHACHA20_POLY1305_KEY_SIZE];
    kdf.finalize_variable(&mut key)?;

    Ok(key)
}

pub fn encrypt(cipher: &XChaCha20Poly1305, data: &[u8], buffer: &mut dyn Buffer) -> Result<XNonce> {
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    cipher
        .encrypt_in_place(&nonce, data, buffer)
        .map_err(|e| anyhow!(e))?;

    Ok(nonce)
}

pub fn decrypt(
    cipher: &XChaCha20Poly1305,
    nonce: &XNonce,
    data: &[u8],
    buffer: &mut dyn Buffer,
) -> Result<()> {
    cipher
        .decrypt_in_place(nonce, data, buffer)
        .map_err(|e| anyhow!(e))
}
