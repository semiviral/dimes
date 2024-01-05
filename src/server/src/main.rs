#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate sqlx;

mod api;
mod cfg;
mod storage;
mod tcp;

use anyhow::Result;
use once_cell::sync::{Lazy, OnceCell};
use std::{collections::BTreeMap, path::PathBuf};
use tokio::{
    net::TcpListener,
    sync::{Mutex, RwLock},
};
use tokio_util::sync::CancellationToken;
use tracing::Level;
use uuid::Uuid;

static TEMP_DIR: Lazy<PathBuf> = Lazy::new(std::env::temp_dir);
static PEER_TOKENS: Mutex<BTreeMap<Uuid, CancellationToken>> = Mutex::const_new(BTreeMap::new());

static PGPOOL: RwLock<OnceCell<sqlx::PgPool>> = RwLock::const_new(OnceCell::new());

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
    connect_db().await?;
    listen().await?;

    Ok(())
}

#[instrument]
async fn connect_db() -> Result<()> {
    static MIGRATOR: sqlx::migrate::Migrator = migrate!();

    let connect_str = cfg::get().db.url.as_str();

    debug!("Instantiating database connection pool: {connect_str:?}");
    event!(Level::DEBUG, db_url = connect_str);
    let pgpool = sqlx::PgPool::connect(connect_str).await?;
    debug!("Running migrations on database...");
    MIGRATOR.run(&pgpool).await?;
    debug!("Migrations complete.");

    let pgpool_rw = PGPOOL.write().await;
    pgpool_rw.set(pgpool).unwrap();

    debug!("Finished connecting to database.");
    Ok(())
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
