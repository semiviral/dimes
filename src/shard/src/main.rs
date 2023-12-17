#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cli;
// mod routes;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use lib::{
    crypto,
    message::{receive_message, send_message, Message},
};
use redis::aio::Connection;
use tokio::sync::{Mutex, OnceCell, SetError};

const CHUNK_SIZE: usize = 512_000;

static ECDH_KEY: OnceCell<crypto::Key> = OnceCell::const_new();
static REDIS_CONN: Mutex<OnceCell<Connection>> = Mutex::const_new(OnceCell::const_new());

#[tokio::main]
async fn main() {
    let args = cli::Arguments::parse();
    println!("{args:?}");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    start(&args)
        .await
        .expect("shard server unexpectedly stopped");

    std::process::exit(1)
}

async fn start(args: &cli::Arguments) -> Result<()> {
    //connect_redis(&args.db_url).await?;

    let mut socket = tokio::net::TcpStream::connect((args.bind_address, args.bind_port)).await?;
    debug!(
        "Successfully connected to endpoint: {}:{}",
        args.bind_address, args.bind_port
    );
    trace!("Negotiating ECDH shared secret with server...");
    let key = crypto::ecdh_handshake(&mut socket).await?;

    let mut stream = tokio::io::BufStream::new(socket);
    let Message::Ping { stamp } = receive_message(&mut stream, &key).await? else {
        bail!("Server started correspondence with unexpected message (expected ping).")
    };

    send_message(&mut stream, &key, Message::Pong { restamp: stamp }).await?;

    println!("all done");

    loop {
        std::thread::sleep(Duration::from_millis(10));
    }
}

async fn connect_redis(db_url: &str) -> Result<()> {
    let client = redis::Client::open(db_url)?;
    let connection = client.get_async_connection().await?;

    let redis_conn = REDIS_CONN.lock().await;
    redis_conn.set(connection).map_err(|err| match err {
        SetError::AlreadyInitializedError(_) => anyhow!("redis connection is already established"),
        SetError::InitializingError(_) => anyhow!("redis connection is establishing"),
    })?;

    Ok(())
}
