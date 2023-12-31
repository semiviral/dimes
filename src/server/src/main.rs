#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;
mod http;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio_postgres::{tls::NoTlsStream, NoTls};
use tokio_util::sync::CancellationToken;
use tracing::Level;

fn agent() -> String {
    format!(
        "{}/{}",
        String::from("dimese-server"),
        env!("CARGO_PKG_VERSION")
    )
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
        _ = ctoken.cancelled() => { Ok(()) }
        result = listen_http(&ctoken) => { result }
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
