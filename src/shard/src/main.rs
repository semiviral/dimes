#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cli;

use anyhow::Result;
use clap::Parser;
use lib::{
    crypto::{self, Key},
    message::{receive_chunk, receive_message, send_message, Message},
};
use redis::aio::Connection;
use std::time::Duration;
use tokio::{
    io::{AsyncRead, AsyncWrite, BufStream},
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
    let args = cli::Arguments::parse();
    println!("{args:?}");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    match start(&args).await {
        Ok(()) => info!("Shard has reached safe shutdown point."),
        Err(err) => error!("Shard has encountered an unrecoverable error: {err:?}"),
    }
}

async fn start(args: &cli::Arguments) -> Result<()> {
    //let redis = connect_redis(&args.db_url).await?;

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

    listen_server( stream, key).await?;

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

    let Message::Ping { stamp } = receive_message(&mut stream, &key).await? else {
        bail!("Server started correspondence with unexpected message (expected ping).")
    };

    send_message(&mut stream, &key, Message::Pong { restamp: stamp }).await?;
    // Send info block to server.
    send_message(
        &mut stream,
        &key,
        Message::Info {
            version: version(),
            agent: agent(),
            max_chunks: args.max_chunks,
        },
    )
    .await?;

    Ok((stream, key))
}

async fn listen_server<S: AsyncRead + AsyncWrite + Unpin>(
    //_redis: Connection,
    mut stream: S,
    key: Key,
) -> Result<()> {
    loop {
        match receive_message(&mut stream, &key).await? {
            Message::Ping { stamp } => {
                send_message(&mut stream, &key, Message::Pong { restamp: stamp }).await?
            }

            Message::PrepareStore { id: _ } => {
                let chunk = receive_chunk(&mut stream, &key).await?;
                info!("{chunk:X?}");
            }

            message => error!("Unexpected message, cannot cope: {message:?}"),
        }
    }
}
