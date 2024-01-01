#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod api;
mod cfg;
mod storage;
mod tcp;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::Level;
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
    //connect_db().await?;
    listen().await?;

    Ok(())
}

async fn connect_db() -> Result<()> {
    // event!(Level::DEBUG, url = &cfg::get().db.url);

    // let (_client, _connection) =
    //     tokio_postgres::connect(&cfg::get().db.url, tokio_postgres::NoTls).await?;

    todo!()
}

async fn listen() -> Result<()> {
    let shard_bind = cfg::get().bind.shard;
    event!(Level::DEBUG, ip = %shard_bind.ip(), port = shard_bind.port());
    let shard_listener = TcpListener::bind(shard_bind).await?;

    let http_bind = cfg::get().bind.http;
    event!(Level::DEBUG, ip = %http_bind.ip(), port = http_bind.port());
    let http_listener = TcpListener::bind(http_bind).await?;

    let ctoken = CancellationToken::new();

    tokio::select! {
        _ = tcp::accept_connections(shard_listener, &ctoken) => { std::process::exit(-100) }
        _ = api::accept_connections(http_listener, &ctoken) => { std::process::exit(-200) }

        _ = ctoken.cancelled() => { Ok(()) }
    }
}
