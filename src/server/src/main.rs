#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod http;
mod tcp;

use std::collections::BTreeMap;
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
    let listener = TcpListener::bind("127.0.0.1:3088").await.unwrap();
    debug!("Server is listening on 127.0.0.1:3088");
    let ctoken = CancellationToken::new();

    tokio::select! {
        _ = ctoken.cancelled() => { std::process::exit(0) }
        _ = tcp::accept_connections(listener, &ctoken) => { std::process::exit(-100) }
        _ = http::accept_connections() => { std::process::exit(-200) }
    }
}
