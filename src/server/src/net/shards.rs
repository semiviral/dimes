use crate::{cfg, PEER_CTOKENS};
use anyhow::Result;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use lib::{
    error::unexpected_message,
    net::{ping_pong, receive_message, Message, MAX_TIMEOUT, MESSAGE_TIMEOUT},
    ShardInfo,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::BufStream,
    net::{TcpListener, TcpStream},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::Level;
use uuid::Uuid;

#[instrument(skip(listener, ctoken))]
pub async fn accept_connections(listener: TcpListener, ctoken: &CancellationToken) -> Result<()> {
    loop {
        trace!("Waiting to accept shard...");
        let (peer_socket, peer_address) = listener.accept().await?;
        let peer_ctoken = ctoken.child_token();

        tokio::spawn(async move {
            let (peer_stream, peer_key, peer_id) =
                spawn_peer(peer_address, peer_socket, &peer_ctoken)
                    .await
                    .expect("error spawning peer");

            listen_peer(peer_id, peer_ctoken, peer_stream, peer_key)
                .await
                .expect("error listening to peer");
        });
    }
}

#[instrument(skip(socket, ctoken))]
async fn spawn_peer(
    address: SocketAddr,
    socket: TcpStream,
    ctoken: &CancellationToken,
) -> Result<(BufStream<TcpStream>, Arc<XChaCha20Poly1305>, Uuid)> {
    let mut stream = BufStream::new(socket);
    let cipher = {
        let key = lib::crypto::ecdh_handshake(&mut stream).await?;
        let cipher = XChaCha20Poly1305::new(&key.into());

        Arc::new(cipher)
    };

    lib::net::negotiate_hello(&mut stream, &cipher).await?;

    let info = match receive_message(&mut stream, &cipher, MESSAGE_TIMEOUT).await? {
        Message::ShardInfo { id, agent, chunks } => Ok(ShardInfo::new(id, agent, chunks)),
        message => unexpected_message("Message::Info", message),
    }?;

    event!(Level::DEBUG, id = ?info.id(), agent = ?info.agent(), chunks = ?info.chunks());

    let mut peer_ctokens = PEER_CTOKENS.lock().await;
    peer_ctokens.insert(info.id(), ctoken.clone());
    drop(peer_ctokens);

    let pg_pool_read = crate::DB_STORE.read().await;
    pg_pool_read.get().unwrap().add_shard(&info).await?;
    drop(pg_pool_read);

    debug!("Connected.");

    Ok((stream, cipher, info.id()))
}

#[instrument(skip(ctoken, stream, cipher))]
async fn listen_peer(
    id: Uuid,
    ctoken: CancellationToken,
    mut stream: BufStream<TcpStream>,
    cipher: impl AsRef<XChaCha20Poly1305>,
) -> Result<()> {
    let ping_wait = Duration::from_millis(cfg::get().interval.ping);

    'a: loop {
        tokio::select! {
            _ = ctoken.cancelled() => break 'a,

            _ = sleep(ping_wait) => ping_pong(&mut stream, &cipher).await,

            message = receive_message(&mut stream, &cipher, MAX_TIMEOUT) => {
                match message {
                    Ok(Message::ShardShutdown) => break 'a,

                    Ok(message) => unexpected_message("Message::ShardShutdown", message),

                    Err(err) => bail!("Error reading message from pipe: {err:?}"),
                }
            }
        }?;
    }

    Ok(())
}
