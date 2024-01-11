use crate::PEER_TOKENS;
use anyhow::Result;
use lib::{
    crypto::Key,
    error::unexpected_message,
    net::{ping_pong, receive_message, types::ShardInfo, Message, MESSAGE_TIMEOUT},
};
use std::{net::SocketAddr, time::Duration};
use tokio::{
    io::BufStream,
    net::{TcpListener, TcpStream},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};
use uuid::Uuid;

#[instrument(skip(listener, ctoken))]
pub async fn accept_connections(listener: TcpListener, ctoken: &CancellationToken) -> Result<()> {
    loop {
        trace!("Waiting to accept shard...");
        let (peer_socket, peer_address) = listener.accept().await?;
        let peer_id = Uuid::now_v7();
        let peer_ctoken = ctoken.child_token();

        event!(Level::DEBUG, ip = %peer_address.ip(), port = peer_address.port(), id = %peer_id);

        tokio::spawn(async move {
            let (peer_stream, peer_key, peer_info) = spawn_peer(peer_id, peer_socket, &peer_ctoken)
                .instrument(span!(Level::DEBUG, "spawn peer", id = %peer_id))
                .await
                .expect("failed spawning peer");

            listen_peer(peer_id, peer_ctoken, peer_stream, peer_address, peer_key)
                .instrument(span!(Level::DEBUG, "listen peer", id = %peer_info.id(), agent = %peer_info.agent()))
                .await
        });
    }
}

async fn spawn_peer(
    peer_id: Uuid,
    peer_socket: TcpStream,
    peer_ctoken: &CancellationToken,
) -> Result<(BufStream<TcpStream>, Key, ShardInfo)> {
    let mut peer_tokens = PEER_TOKENS.lock().await;
    peer_tokens.insert(peer_id, peer_ctoken.clone());
    drop(peer_tokens);

    let mut peer_stream = BufStream::new(peer_socket);
    let peer_key = lib::crypto::ecdh_handshake(&mut peer_stream)
        .await
        .map_err(|err| anyhow!("[{peer_id}] Error during handshake: {err:?}"))?;

    lib::net::negotiate_hello(&mut peer_stream, &peer_key).await?;

    let peer_info = match receive_message(&mut peer_stream, &peer_key, MESSAGE_TIMEOUT).await? {
        Message::Info(info) => Ok(info),

        message => unexpected_message("Message::Info", message),
    }
    .unwrap();

    event!(
        Level::DEBUG,
        peer.id = %peer_info.id().to_string(),
        peer.agent = %&peer_info.agent(),
        peer.chunks = peer_info.max_chunks()
    );

    let pg_pool_read = crate::DB_STORE.read().await;
    pg_pool_read
        .get()
        .unwrap()
        .add_shard(peer_info.clone())
        .await?;
    drop(pg_pool_read);

    debug!("Connected.");

    // TODO use a string cache for the agent infos
    Ok((peer_stream, peer_key, peer_info))
}

async fn listen_peer(
    _id: Uuid,
    ctoken: CancellationToken,
    mut stream: BufStream<TcpStream>,
    _address: SocketAddr,
    key: Key,
) -> Result<()> {
    const PING_WAIT: Duration = Duration::from_secs(30);

    'a: loop {
        tokio::select! {
            _ = ctoken.cancelled() => break 'a,
            _ = sleep(PING_WAIT) => ping_pong(&mut stream, &key).await,

            message = receive_message(&mut stream, &key, None) => {
                match message {
                    Ok(message) => {
                       todo!("handle {message:?}")
                    }

                    Err(err) => {
                       bail!("Error reading message from pipe: {err:?}")
                    }
                }
            }
        }?;
    }

    Ok(())
}
