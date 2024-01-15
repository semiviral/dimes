use anyhow::Result;
use blake2::digest::{Update, VariableOutput};
use chacha20poly1305::{aead::Aead, AeadCore, KeyInit, Nonce, XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use tokio::io::{AsyncRead, AsyncWrite};
use x25519_dalek::{EphemeralSecret, PublicKey};

const XCHACHA20_POLY1305_KEY_SIZE: usize = 32;
const XCHACHA20_POLY1305_NONCE_SIZE: usize = 24;

pub type Key = [u8; XCHACHA20_POLY1305_KEY_SIZE];

pub async fn ecdh_handshake<R: AsyncRead + AsyncWrite + Unpin>(mut stream: R) -> Result<Key> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let private_key = EphemeralSecret::random_from_rng(&mut OsRng);
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




pub fn encrypt(key: &Key, data: &[u8]) -> Result<(XNonce, Box<[u8]>)> {

    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let cipher = XChaCha20Poly1305::new(key.into());
    let encrypted_data = cipher
        .encrypt(&nonce, data)
        .map_err(|err| anyhow!("Encryption error: {}", err))?;

    Ok((nonce.into(), encrypted_data.into_boxed_slice()))
}

pub fn decrypt(key: &Key, nonce: &XNonce, data: &[u8]) -> Result<Box<[u8]>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let decrypted_data = cipher
        .decrypt(nonce.into(), data)
        .map_err(|err| anyhow!("Decryption error: {}", err))?;

    Ok(decrypted_data.into_boxed_slice())
}
