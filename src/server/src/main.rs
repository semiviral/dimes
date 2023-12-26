#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod http;
mod tcp;

use anyhow::Result;
use config::Config;
use std::{collections::BTreeMap, net::SocketAddr};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

static PEER_TOKENS: Mutex<BTreeMap<Uuid, CancellationToken>> = Mutex::const_new(BTreeMap::new());

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
    let config = load_config()?;

    let shard_listener = TcpListener::bind(config.get::<SocketAddr>("shard.bind")?).await?;
    debug!("Bound shard listener on {}.", shard_listener.local_addr()?);

    let http_listener = TcpListener::bind(config.get::<SocketAddr>("http.bind")?).await?;
    debug!("Bound HTTP listener on {}.", http_listener.local_addr()?);

    let ctoken = CancellationToken::new();

    tokio::select! {
        _ = tcp::accept_connections(shard_listener, &ctoken) => { std::process::exit(-100) }
        _ = http::accept_connections(http_listener, &ctoken) => { std::process::exit(-200) }

        _ = ctoken.cancelled() => { Ok(()) }
    }
}

fn load_config() -> Result<Config> {
    let config = config::Config::builder()
        .add_source(config::Environment::with_prefix("DIMESE").separator("_"))
        .build()?;

    Ok(config)
}
