use anyhow::Result;
use lib::{
    bstr::BStr,
    net::{Connection, Message},
};
use once_cell::sync::Lazy;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    time::timeout,
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

    let stream = timeout(Duration::from_secs(5), TcpStream::connect(&*addrs)).await??;

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

        listen(Connection::new(stream)).await
    } else {
        listen(Connection::new(stream)).await
    }
}

async fn listen<IO: AsyncRead + AsyncWrite + Unpin>(mut connection: Connection<IO>) -> Result<()> {
    static TIMEOUT: Lazy<Duration> = Lazy::new(|| cfg::get().message_timeout());

    if let Message::AssignId { id } = recv_timeout(&mut connection, *TIMEOUT).await? {
        connection.send(Message::Ok, true).await?;
        todo!("set ID")
    }

    if let Message::ServerInfo { agent } = recv_timeout(&mut connection, *TIMEOUT).await? {
        send_timeout(&mut connection, Message::Ok, true, *TIMEOUT).await?;
        todo!("set server info")
    }

    let shard_info = Message::ShardInfo {
        chunks: cfg::get().storage().chunks(),
        agent: BStr::new(crate::agent_str()),
    };
    send_timeout(&mut connection, shard_info, true, *TIMEOUT).await?;

    loop {
        match connection.recv().await? {
            Message::Ok => {}

            Message::Ping => {
                send_timeout(&mut connection, Message::Pong, true, *TIMEOUT).await?;
            }

            Message::Pong => {
                debug!("Server sent unexpected pong. Ignoring.");
            }

            Message::ShardStore { chunk } => todo!(),
            Message::ShardRetrieve { id } => todo!(),
            Message::ShardChunkExists { id } => todo!(),

            message => {
                error!("Unexpected message (ignoring): {:?}", message);
            }
        }
    }
}

async fn send_timeout<IO: AsyncRead + AsyncWrite + Unpin>(
    connection: &mut Connection<IO>,
    message: Message,
    flush: bool,
    timeout: Duration,
) -> Result<()> {
    tokio::time::timeout(timeout, async { connection.send(message, flush).await }).await??;

    Ok(())
}

async fn recv_timeout<IO: AsyncRead + AsyncWrite + Unpin>(
    connection: &mut Connection<IO>,
    timeout: Duration,
) -> Result<Message> {
    let message = tokio::time::timeout(timeout, async { connection.recv().await }).await??;

    Ok(message)
}
