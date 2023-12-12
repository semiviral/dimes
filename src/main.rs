#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cli;
mod crypto;
// mod routes;

use anyhow::Result;
use clap::Parser;
use redis::aio::Connection;
use tokio::sync::{Mutex, OnceCell, SetError};

const CHUNK_SIZE: usize = 512_000;

static ECDH_KEY: OnceCell<crypto::EcdhKey> = OnceCell::const_new();
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

#[repr(C)]
pub enum Message {
    a,
}

async fn start(args: &cli::Arguments) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    connect_redis(&args.db_url).await?;

    let mut socket = tokio::net::TcpStream::connect((args.bind_address, args.bind_port)).await?;
    debug!(
        "Successfully onnected to endpoint: {}:{}",
        args.bind_address, args.bind_port
    );

    trace!("Negotiating ECDH shared secret with server...");
    let ecdh_key = crypto::ecdh_handshake(&mut socket).await?;
    ECDH_KEY.set(ecdh_key)?;

    println!("yay");

    Ok(())
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
