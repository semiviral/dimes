#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;

use anyhow::Result;
use lib::{
    crypto::Key,
    message::{receive_message, send_message, Message, MESSAGE_TIMEOUT},
};
use std::time::Duration;
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream},
    net::TcpStream,
};

fn version() -> String {
    String::from(env!("CARGO_PKG_VERSION"))
}

fn agent() -> String {
    String::from("dimese-shard")
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

    lib::message::negotiate_hello(&mut stream, &key).await?;

    // Send info block to server.
    send_message(
        &mut stream,
        &key,
        Message::Info {
            version: version(),
            agent: agent(),
            max_chunks: cfg::get().storage.max,
        },
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    Ok((stream, key))
}

async fn listen_server<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S, key: Key) -> Result<()> {
    loop {
        match receive_message(&mut stream, &key, None).await? {
            Message::Ping => {
                send_message(&mut stream, &key, Message::Pong, MESSAGE_TIMEOUT, true).await?
            }

            Message::PrepareStore { id } => {
                let file_path = cfg::get().storage.directory.join(&id.to_string());
                let file = File::options()
                    .create_new(true)
                    .write(true)
                    .read(false)
                    .open(&file_path)
                    .await?;

                let mut file_buf = BufStream::new(file);
                for _ in 0..lib::message::CHUNK_PARTS {
                    match receive_message(&mut stream, &key, MESSAGE_TIMEOUT).await? {
                        Message::ChunkPart(part) => {
                            file_buf.write_all(&part).await?;
                        }

                        message => {
                            bail!("Expected chunk part, got: {message:?}")
                        }
                    }
                }

                file_buf.flush().await?;
            }

            Message::PrepareStock { id } => {
                let file_path = cfg::get().storage.directory.join(&id.to_string());
                let file = File::options()
                    .create(false)
                    .write(false)
                    .read(true)
                    .open(&file_path)
                    .await?;

                let mut file_buf = BufStream::new(file);
                let mut part_buf = [0u8; lib::message::CHUNK_PART_SIZE];
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
