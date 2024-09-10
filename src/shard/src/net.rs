use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use lib::net::Message;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
    TlsConnector,
};

use crate::cfg;

pub async fn connect() -> Result<()> {
    let addrs = crate::cfg::get()
        .remote()
        .to_socket_addrs()
        .expect("remote address cannot be resolved")
        .collect::<Box<[SocketAddr]>>();
    let stream = TcpStream::connect(&*addrs).await?;

    if cfg::get().use_tls() {
        let mut root_cert_store = RootCertStore::empty();
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

        listen(stream).await
    } else {
        listen(stream).await
    }
}

async fn listen(mut stream: impl AsyncRead + AsyncWrite + Unpin) -> Result<()> {
    Message::ShardInfo {
        chunks: cfg::get().storage().chunks(),
        agent: crate::agent_str().to_string(),
    }
    .send(&mut stream);

    // TODO receive server info

    if let Message::Info = Message::recv(stream)
    

    loop {
        match Message::recv(&mut stream).await? {
            Message::Ok => {}

            Message::Ping => {
                Message::Pong.send(&mut stream).await?;
            }

            Message::Pong => {
                debug!("Server sent unexpected pong. Ignoring.");
            }

            Message::ShardInfo { id, chunks, agent } => todo!(),

            Message::ShardStore { chunk } => todo!(),

            Message::ShardRetrieve { id } => todo!(),

            Message::ShardChunkExists { id } => todo!(),
        }
    }
}
