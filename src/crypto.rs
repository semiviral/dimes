use blake2::digest::{Update, VariableOutput};
use rand::{rngs::OsRng, RngCore};
use x25519_dalek::*;

const XCHACHA20_POLY1305_KEY_SIZE: usize = 32;
const XCHACHA20_POLY1305_NONCE_SIZE: usize = 24;

#[derive(Debug)]
pub struct EcdhKey {
    pub nonce: [u8; XCHACHA20_POLY1305_NONCE_SIZE],
    pub key: Box<[u8]>,
}

pub async fn ecdh_handshake(socket: &mut tokio::net::TcpStream) -> anyhow::Result<EcdhKey> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut rng = OsRng;
    let mut nonce = [0u8; XCHACHA20_POLY1305_NONCE_SIZE];
    rng.fill_bytes(&mut nonce);

    let private_key = EphemeralSecret::random_from_rng(rng);
    let public_key = PublicKey::from(&private_key);

    socket.write_all(public_key.as_bytes()).await?;
    let mut peer_public_key = [0u8; 32];
    socket.read_exact(&mut peer_public_key).await?;
    let peer_public_key = PublicKey::from(peer_public_key);

    let dh_secret = private_key.diffie_hellman(&peer_public_key);
    let mut kdf = blake2::Blake2bVar::new(XCHACHA20_POLY1305_KEY_SIZE).unwrap();
    kdf.update(dh_secret.as_bytes());
    let key = kdf.finalize_boxed();

    Ok(EcdhKey { nonce, key })
}
