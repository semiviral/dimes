#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

use anyhow::Result;
use lib::{
    crypto::Key,
    message::{ping_peer, receive_message, send_chunk, Message, CHUNK_SIZE, MESSAGE_TIMEOUT},
};
use rand::{rngs::OsRng, RngCore};
use std::{collections::BTreeMap, net::SocketAddr, time::Duration};
use tokio::{
    io::BufStream,
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};
use uuid::Uuid;

static PEER_TOKENS: Mutex<BTreeMap<Uuid, CancellationToken>> = Mutex::const_new(BTreeMap::new());

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    info!("Starting server...");
    let listener = TcpListener::bind("127.0.0.1:3088").await.unwrap();
    debug!("Server is listening on 127.0.0.1:3088");
    let ctoken = CancellationToken::new();

    tokio::select! {
        _ = ctoken.cancelled() => { std::process::exit(0) }
        _ = accept_connections(listener, &ctoken) => { std::process::exit(-1) }
    }
}

async fn accept_connections(listener: TcpListener, ctoken: &CancellationToken) -> Result<()> {
    loop {
        trace!("Server is waiting to accept a socket.");
        let (peer_socket, peer_address) = listener.accept().await?;
        let peer_id = Uuid::now_v7();
        let peer_ctoken = ctoken.child_token();
        debug!("Accepted socket [{peer_id}]: {peer_address}");

        tokio::spawn(
            async move {
                spawn_peer(peer_id, peer_ctoken, peer_socket, peer_address)
                    .await
                    .expect("failed spawning peer")
            }
            .instrument(info_span!("peer", peer_id = %peer_id)),
        );
    }
}

async fn spawn_peer(
    peer_id: Uuid,
    peer_ctoken: CancellationToken,
    peer_socket: TcpStream,
    peer_address: SocketAddr,
) -> Result<()> {
    let mut peer_tokens = PEER_TOKENS.lock().await;
    peer_tokens.insert(peer_id, peer_ctoken.clone());
    drop(peer_tokens);

    let mut peer_stream = BufStream::new(peer_socket);
    let peer_key = lib::crypto::ecdh_handshake(&mut peer_stream)
        .await
        .map_err(|err| anyhow!("[{peer_id}] Error during handshake: {err:?}"))?;

    ping_peer(&mut peer_stream, &peer_key).await?;

    debug!("[{peer_id}] Peer correctly restamped initial ping.");

    let Message::Info {
        agent,
        version,
        max_chunks,
    } = receive_message(&mut peer_stream, &peer_key, MESSAGE_TIMEOUT).await?
    else {
        bail!("expected Info message")
    };

    let mut chunk = vec![0u8; CHUNK_SIZE].into_boxed_slice();
    OsRng.fill_bytes(&mut chunk);
    send_chunk(&mut peer_stream, &peer_key, Uuid::now_v7(), &chunk).await?;

    event!(
        Level::DEBUG,
        peer.id = %peer_id.to_string(),
        peer.agent = %&agent,
        peer.version = %&version,
        peer.chunks = max_chunks
    );

    listen_peer(peer_id, peer_ctoken, peer_stream, peer_address, peer_key).await
}

async fn listen_peer(
    _id: Uuid,
    ctoken: CancellationToken,
    mut stream: BufStream<TcpStream>,
    _address: SocketAddr,
    key: Key,
) -> Result<()> {
    const PING_WAIT: Duration = Duration::from_secs(10);

    'a: loop {
        tokio::select! {
            _ = ctoken.cancelled() => break 'a,
            _ = sleep(PING_WAIT) => ping_peer(&mut stream, &key).await,

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
