#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;
mod pools;

use anyhow::Result;
use lib::{
    crypto::Key,
    net::{
        receive_message, send_message,
        types::{ChunkHash, ShardInfo},
        Message, MESSAGE_TIMEOUT,
    },
};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, ops::Deref, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, RwLock,
    },
};
use uuid::Uuid;

static ID: Lazy<Uuid> = Lazy::new(Uuid::now_v7);

fn agent() -> String {
    format!("dimese-shard/{}", env!("CARGO_PKG_VERSION"))
}

fn info() -> ShardInfo {
    ShardInfo::new(*ID, agent(), cfg::get().storage.chunks).unwrap()
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    dotenvy::dotenv().unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    match run().await {
        Ok(()) => info!("Shard has reached safe shutdown point."),
        Err(err) => error!("Shard has encountered an unrecoverable error: {err:?}"),
    }
}

async fn run() -> Result<()> {
    trace!("Attempting to connect to server...");

    let (stream, key) = connect_server().await?;

    use tokio::sync::mpsc::channel;
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let writer = BufWriter::new(writer);

    let (send_queue, recv_queue) = channel(cfg::get().queuing.send);
    let send_queue = Arc::new(send_queue);

    tokio::spawn(process_writes(writer, recv_queue, key));

    loop {
        let message = receive_message(&mut reader, &key, None).await?;
        tokio::spawn(process_message(Arc::clone(&send_queue), message));
    }
}

async fn connect_server() -> Result<(TcpStream, Key)> {
    let server_addr = cfg::get().server.address;

    let mut stream = loop {
        debug!("Attempting to connect to server @{server_addr}");

        match TcpStream::connect(server_addr).await {
            Ok(stream) => break stream,

            Err(err) => {
                error!("Error connecting to server: {err:?}");
                error!("Waiting 10 seconds to try again.");
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    };

    trace!("Negotiating ECDH shared secret with server...");
    let key = lib::crypto::ecdh_handshake(&mut stream).await?;

    lib::net::negotiate_hello(&mut stream, &key).await?;

    // Send info block to server.
    send_message(
        &mut stream,
        &key,
        Message::ShardInfo(info()),
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    Ok((stream, key))
}

static STORING_CHUNKS: RwLock<BTreeMap<ChunkHash, Mutex<Box<[u8]>>>> =
    RwLock::const_new(BTreeMap::new());

#[allow(clippy::large_enum_variant)]
pub enum WriteCommand {
    Send(Message),
    Flush,
}

#[instrument(skip(writer, recv_queue, key))]
async fn process_writes(
    mut writer: impl AsyncWrite + Unpin,
    mut recv_queue: Receiver<WriteCommand>,
    key: Key,
) {
    while let Some(command) = recv_queue.recv().await {
        match command {
            WriteCommand::Send(message) => {
                send_message(&mut writer, &key, message, MESSAGE_TIMEOUT, false)
                    .await
                    .expect("failed to write message to stream")
            }

            WriteCommand::Flush => writer.flush().await.expect("failed to flush write stream"),
        }
    }
}

#[instrument(skip(send_queue))]
async fn process_message(send_queue: impl Deref<Target = Sender<WriteCommand>>, message: Message) {
    match message {
        Message::Ping => send_queue
            .send(WriteCommand::Send(Message::Pong))
            .await
            .expect("failed to send ping"),

            Message::PrepareStore { hash } => {
                
            }

        message => error!("Unexpected message, cannot cope: {message:?}"),
    }
}
