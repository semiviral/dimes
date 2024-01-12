#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;

use anyhow::Result;
use lib::{
    crypto::Key,
    error::unexpected_message,
    net::{
        receive_message, send_message,
        types::{ChunkHash, ShardInfo},
        Message, MESSAGE_TIMEOUT,
    },
};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, time::Duration};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream},
    net::TcpStream,
    sync::{Mutex, RwLock},
};
use uuid::Uuid;

static ID: Lazy<Uuid> = Lazy::new(Uuid::now_v7);

fn agent() -> String {
    format!("dimese-shard/{}", env!("CARGO_PKG_VERSION"))
}

fn info() -> ShardInfo {
    ShardInfo::new(*ID, agent(), cfg::get().storage.chunks).unwrap()
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    dotenvy::dotenv().unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    match start().await {
        Ok(()) => info!("Shard has reached safe shutdown point."),
        Err(err) => error!("Shard has encountered an unrecoverable error: {err:?}"),
    }
}

async fn start() -> Result<()> {
    trace!("Attempting to connect to server...");

    let (stream, key) = {
        let mut reconnections = 5;
        'a: loop {
            match connect_server().await {
                Ok(ok) => break 'a ok,

                Err(err) => {
                    error!("Error connecting to server: {err:?}");

                    if reconnections == 0 {
                        error!("Retried too many times; failed to connect to server.");

                        std::process::exit(404);
                    } else {
                        reconnections -= 1;
                    }

                    error!("Retrying connection in 10 seconds.");

                    tokio::time::sleep(Duration::from_secs(10)).await;

                    continue 'a;
                }
            }
        }
    };

    listen_server(stream, key).await?;

    Ok(())
}

async fn connect_server() -> Result<(BufStream<TcpStream>, Key)> {
    let server_addr = cfg::get().server.address;
    debug!("Attempting to connect to server @{server_addr}");
    let mut stream = BufStream::new(TcpStream::connect(server_addr).await?);

    trace!("Negotiating ECDH shared secret with server...");
    let key = lib::crypto::ecdh_handshake(&mut stream).await?;

    lib::net::negotiate_hello(&mut stream, &key).await?;

    // Send info block to server.
    send_message(
        &mut stream,
        &key,
        Message::ShardInfo(info()),
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    Ok((stream, key))
}

static STORING_CHUNKS: RwLock<BTreeMap<ChunkHash, Mutex<Box<[u8]>>>> =
    RwLock::const_new(BTreeMap::new());
static STOCKING_CHUNKS: RwLock<BTreeMap<ChunkHash, Mutex<Box<[u8]>>>> =
    RwLock::const_new(BTreeMap::new());

async fn listen_server<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, key: Key) -> Result<()> {
    loop {
        match receive_message(&mut stream, &key, None).await? {
            Message::Ping => {
                send_message(&mut stream, &key, Message::Pong, MESSAGE_TIMEOUT, true).await?
            }

            Message::PrepareStore { hash } => {
                let storing_chunks = STORING_CHUNKS.read().await;
                
                if storing_chunks.contains_key(&hash) {
                    send_message(&mut stream, &key, Message, timeout, true).await?;

                }

                
                let storing_chunks = STORING_CHUNKS.write().await;


                for _ in 0..lib::net::CHUNK_PARTS {
                    match receive_message(&mut stream, &key, MESSAGE_TIMEOUT).await? {
                        Message::ChunkPart(part) => {
                            file_buf.write_all(&part).await?;
                        }

                        message => unexpected_message("Message::ChunkPart", message),
                    }
                }
            }

            Message::PrepareStock { hash: id } => {
                let file_path = cfg::get().storage.directory.join(&id.to_string());
                let file = File::options()
                    .create(false)
                    .write(false)
                    .read(true)
                    .open(&file_path)
                    .await?;

                let mut file_buf = BufStream::new(file);
                let mut part_buf = [0u8; lib::net::CHUNK_PART_SIZE];
                loop {
                    let bytes_read = file_buf.read_exact(&mut part_buf).await?;

                    if bytes_read == 0 {
                        break;
                    }

                    assert_eq!(bytes_read, part_buf.len(), "file is unexpected size");

                    send_message(
                        &mut stream,
                        &key,
                        Message::ChunkPart(part_buf),
                        MESSAGE_TIMEOUT,
                        false,
                    )
                    .await?;
                }

                stream.flush().await?;
            }

            message => error!("Unexpected message, cannot cope: {message:?}"),
        }
    }
}
