use std::{net::ToSocketAddrs, sync::Arc};

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

    connect().unwrap();

    info!("Reached a safe shutdown point.");
}

async fn connect() -> Result<()> {
    let addrs = cfg::get()
        .remote()
        .to_socket_addrs()
        .expect("remote address cannot be resolved");
    let stream = TcpStream::connect(&addrs).await?;

    if cfg::get().use_tls() {
        let root_cert_store = RootCertStore::empty();
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let tls_connector = TlsConnector::from(Arc::new(config));
        let dns_name = ServerName::try_from(cfg::get().remote()).expect("not a valid remote host");

        let stream = tls_connector
            .connect(dns_name, stream)
            .await
            .expect("TLS connect did not succeed");

        listen(stream)
    } else {
        listen(stream)
    }
}

async fn listen<S: AsyncRead + AsyncWrite + Unpin>(stream: S) -> Result<S> {}
