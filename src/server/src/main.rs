#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;
mod http;
mod storage;
mod tcp;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

static TEMP_DIR: Lazy<PathBuf> = Lazy::new(std::env::temp_dir);

static PEER_TOKENS: Mutex<BTreeMap<Uuid, CancellationToken>> = Mutex::const_new(BTreeMap::new());

fn agent() -> String {
    format!("{}/{}", String::from("Dimese"), env!("CARGO_PKG_VERSION"))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    info!("Starting server...");

    start().await.unwrap();

    std::process::exit(0);
}

async fn start() -> Result<()> {
    let shard_listener = TcpListener::bind(cfg::get().bind.shard).await?;
    debug!("Bound shard listener on {}.", shard_listener.local_addr()?);

    let http_listener = TcpListener::bind(cfg::get().bind.http).await?;
    debug!("Bound HTTP listener on {}.", http_listener.local_addr()?);

    let ctoken = CancellationToken::new();

    tokio::select! {
        _ = tcp::accept_connections(shard_listener, &ctoken) => { std::process::exit(-100) }
        _ = http::accept_connections(http_listener, &ctoken) => { std::process::exit(-200) }

        _ = ctoken.cancelled() => { Ok(()) }
    }
}

async fn connect_db() -> Result<()> {
    let (client, connection) = tokio_postgres::connect("config", tokio_postgres::NoTls).await?;

    
}
