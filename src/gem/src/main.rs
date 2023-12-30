#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;
mod http;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_postgres::{tls::NoTlsStream, NoTls};
use tokio_util::sync::CancellationToken;
use tracing::Level;
use uuid::Uuid;

static TEMP_DIR: Lazy<PathBuf> = Lazy::new(std::env::temp_dir);

static PEER_TOKENS: Mutex<BTreeMap<Uuid, CancellationToken>> = Mutex::const_new(BTreeMap::new());

fn agent() -> String {
    format!(
        "{}/{}",
        String::from("dimese-gem"),
        env!("CARGO_PKG_VERSION")
    )
}

fn info() -> lib::ConnectInfo {
    lib::ConnectInfo { agent: agent() }
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
    // connect_db().await?;

    let ctoken = CancellationToken::new();

    tokio::select! {
        result = listen_http(&ctoken) => { result }
        _ = ctoken.cancelled() => { Ok(()) }
    }
}

#[instrument]
async fn connect_db() -> Result<(
    tokio_postgres::Client,
    tokio_postgres::Connection<tokio_postgres::Socket, NoTlsStream>,
)> {
    event!(Level::DEBUG, config = &cfg::get().db.url);

    Ok(tokio_postgres::connect(&cfg::get().db.url, NoTls).await?)
}

#[instrument("listen", skip(ctoken))]
async fn listen_http(ctoken: &CancellationToken) -> Result<()> {
    let http_bind = cfg::get().bind;
    event!(Level::DEBUG, ip = %http_bind.ip(), port = http_bind.port());
    let http_listener = TcpListener::bind(http_bind).await?;

    http::accept_connections(http_listener, ctoken).await?;

    Ok(())
}
