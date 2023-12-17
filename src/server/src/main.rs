#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

use anyhow::Result;
use lib::{
    crypto::Key,
    message::{ping_peer, receive_message},
};
use std::{collections::BTreeMap, net::SocketAddr, time::Duration};
use tokio::{
    io::BufStream,
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::{timeout, Instant},
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

static PEER_TOKENS: Mutex<BTreeMap<Uuid, CancellationToken>> = Mutex::const_new(BTreeMap::new());

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    info!("Starting server...");
    let listener = TcpListener::bind("127.0.0.1:3088").await.unwrap();
    debug!("Server is listening on 127.0.0.1:3000");
    let server_ctoken = CancellationToken::new();

    while timeout(Duration::from_millis(100), server_ctoken.cancelled())
        .await
        .is_err()
    {
        trace!("Server is waiting to accept a socket.");
        let (peer_socket, peer_address) = listener.accept().await.unwrap();
        let peer_id = Uuid::new_v4();
        let peer_ctoken = server_ctoken.child_token();
        debug!("Accepted socket [{peer_id}]: {peer_address}");

        tokio::spawn(async move {
            spawn_peer(peer_id, peer_ctoken, peer_socket, peer_address)
                .await
                .expect("failed spawning peer")
        });
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

    debug!("[{peer_id}] Peer correctly restamped initial ping. Starting listen loop.");
    listen_peer(peer_id, peer_ctoken, peer_stream, peer_address, peer_key).await
}

async fn listen_peer(
    _id: Uuid,
    ctoken: CancellationToken,
    mut stream: BufStream<TcpStream>,
    _address: SocketAddr,
    key: Key,
) -> Result<()> {
    const MSG_WAIT_INTERVAL: Duration = Duration::from_millis(1000);

    let mut last_ping = Instant::now();
    while !ctoken.is_cancelled() {
        let now = Instant::now();
        if (now - last_ping) > Duration::from_secs(10) {
            ping_peer(&mut stream, &key).await?;
            last_ping = now;
        }

        match timeout(MSG_WAIT_INTERVAL, receive_message(&mut stream, &key)).await {
            Ok(Ok(_message)) => {
                todo!("handle message")
            }

            Ok(Err(err)) => bail!(err),

            _ => {}
        }
    }

    Ok(())
}
