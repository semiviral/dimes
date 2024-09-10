use std::{net::ToSocketAddrs, sync::Arc};

use chrono::Duration;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{self, pki_types::ServerName, ClientConfig, RootCertStore},
    TlsConnector,
};

#[macro_use]
extern crate tracing;

mod cfg;
mod storage;
mod net;

fn agent_str() -> &'static str {
    concat!("dimese-shard/", env!("CARGO_PKG_VERSION"))
}

#[tokio::main]
async fn main() {
    use storage::info;
    use tokio::net;

    // Load the environment variables from `.env`.
    dotenvy::dotenv().unwrap();

    // Initialize the async tracing formatter.
    tracing_subscriber::fmt()
        .with_max_level({
            #[cfg(debug_assertions)]
            {
                tracing::Level::TRACE
            }

            #[cfg(not(debug_assertions))]
            {
                tracing::Level::INFO
            }
        })
        .init();

    info!("Begin initializing...");

    trace!("Initializing storage...");
    storage::init();

    trace!("Initializing info...");
    info::init().expect("failed to initialize info");
    debug!("Shard ID: {}", info::get_id());
    debug!("Started: {}", info::get_started_at());

    net::connect().await;

    info!("Reached a safe shutdown point.");
}