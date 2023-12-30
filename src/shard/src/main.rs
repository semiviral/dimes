#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod api;
mod cfg;
mod cli;

use anyhow::Result;
use clap::Parser;

use lib::{
    crypto::{self, Key},
    message::{receive_chunk, receive_message, send_chunk, send_message, Message, MESSAGE_TIMEOUT},
    ConnectInfo, ShardInfo,
};
use once_cell::sync::Lazy;
use redis::{aio::Connection, AsyncCommands};
use reqwest::header::HeaderMap;
use std::time::Duration;
use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
    net::TcpStream,
};

fn agent() -> String {
    format!("dimese-shard/{}", env!("CARGO_PKG_VERSION"))
}

fn info() -> (ConnectInfo, ShardInfo) {
    (
        ConnectInfo { agent: agent() },
        ShardInfo { max_chunks: 128 },
    )
}

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .use_native_tls()
        .build()
        .expect("failed to initialize HTTP client")
});

#[tokio::main]
async fn main() {
    let args = cli::Arguments::parse();
    println!("{args:?}");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    tokio::try_join!(api::start(), announce()).unwrap();

    std::process::exit(0)
}

async fn announce() -> Result<()> {
    let response = HTTP_CLIENT
        .post("http://127.0.0.1:3089/shards/register")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&info())?)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::CREATED => {
            let body = response.bytes().await?;
            let connect_info = serde_json::from_slice::<ConnectInfo>(&body)?;

            info!("Server connection info:\n{connect_info:#?}");

            Ok(())
        }

        status => {
            bail!("Register request failed with status code: {:?}", status)
        }
    }
}

async fn start(args: &cli::Arguments) -> Result<()> {
    let redis = connect_redis(&args.db_url).await?;

    trace!("Attempting to connect to server...");

    let (stream, key) = {
        let mut reconnections = 5;
        'a: loop {
            match connect_server(args).await {
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

    listen_server(redis, stream, key).await?;

    Ok(())
}

async fn connect_redis(db_url: &str) -> Result<Connection> {
    let client = redis::Client::open(db_url)?;
    let connection = client.get_async_connection().await?;

    Ok(connection)
}

async fn connect_server(args: &cli::Arguments) -> Result<(BufStream<TcpStream>, Key)> {
    let bind_address = args.bind_address;
    let bind_port = args.bind_port;

    debug!("Attempting to connect to server @{bind_address}:{bind_port}");
    let socket = tokio::net::TcpStream::connect((bind_address, bind_port)).await?;
    debug!("Connected to server @{bind_address}:{bind_port}");
    let mut stream = BufStream::new(socket);

    trace!("Negotiating ECDH shared secret with server...");
    let key = crypto::ecdh_handshake(&mut stream).await?;

    let Message::Ping { stamp } = receive_message(&mut stream, &key, MESSAGE_TIMEOUT).await? else {
        bail!("Server started correspondence with unexpected message (expected ping).")
    };

    send_message(
        &mut stream,
        &key,
        Message::Pong { restamp: stamp },
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    // Send info block to server.
    send_message(
        &mut stream,
        &key,
        Message::Info {
            version: String::from("Asdasdasd"),
            agent: agent(),
            max_chunks: args.max_chunks,
        },
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    Ok((stream, key))
}

async fn listen_server<S: AsyncRead + AsyncWrite + Unpin>(
    mut redis: Connection,
    mut stream: S,
    key: Key,
) -> Result<()> {
    loop {
        match receive_message(&mut stream, &key, None).await? {
            Message::Ping { stamp } => {
                send_message(
                    &mut stream,
                    &key,
                    Message::Pong { restamp: stamp },
                    MESSAGE_TIMEOUT,
                    true,
                )
                .await?
            }

            Message::PrepareStore { id } => {
                let chunk = receive_chunk(&mut stream, &key).await?;
                redis.hset(id.as_bytes(), "blob", chunk.as_ref()).await?;
            }

            Message::PrepareStock { id } => {
                let chunk: Box<[u8]> = redis.hget(id.as_bytes(), "blob").await?;
                send_chunk(&mut stream, &key, chunk.as_ref()).await?;
            }

            message => error!("Unexpected message, cannot cope: {message:?}"),
        }
    }
}
