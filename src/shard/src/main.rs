use anyhow::Result;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use lib::net::{receive_message, send_message, Message, MAX_TIMEOUT, MESSAGE_TIMEOUT};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;
mod id;
mod storage;

fn agent_str() -> &'static str {
    concat!("dimese-shard/", env!("CARGO_PKG_VERSION"))
}

async fn info_message() -> Message {
    let mut agent = lib::pools::get_string_buf().await;
    agent.push_str(agent_str());

    Message::ShardInfo {
        id: id::get().await,
        agent,
        chunks: cfg::get().storage.chunks,
    }
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

    let (stream, cipher) = connect_server().await?;

    use tokio::sync::mpsc::channel;
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let writer = BufWriter::new(writer);

    let (send_queue, recv_queue) = channel(cfg::get().caching.queues.into());
    let send_queue = Arc::new(send_queue);

    tokio::spawn(process_writes(writer, Arc::clone(&cipher), recv_queue));

    loop {
        let message = receive_message(&mut reader, &cipher, MAX_TIMEOUT).await?;
        tokio::spawn(process_message(Arc::clone(&send_queue), message));
    }
}

async fn connect_server() -> Result<(TcpStream, Arc<XChaCha20Poly1305>)> {
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
    let cipher = {
        let key = lib::crypto::ecdh_handshake(&mut stream).await?;
        let cipher = XChaCha20Poly1305::new(&key.into());

        Arc::new(cipher)
    };

    lib::net::negotiate_hello(&mut stream, &cipher).await?;

    // Send info block to server.
    send_message(
        &mut stream,
        &cipher,
        info_message().await,
        MESSAGE_TIMEOUT,
        true,
    )
    .await?;

    Ok((stream, cipher))
}

#[allow(clippy::large_enum_variant)]
pub enum WriteCommand {
    Send(Message),
    Flush,
}

#[instrument(skip(writer, recv_queue, cipher))]
async fn process_writes<W: AsyncWrite + Unpin, C: AsRef<XChaCha20Poly1305>>(
    mut writer: W,
    cipher: C,
    mut recv_queue: Receiver<WriteCommand>,
) {
    while let Some(command) = recv_queue.recv().await {
        match command {
            WriteCommand::Send(message) => send_message(
                &mut writer,
                cipher.as_ref(),
                message,
                MESSAGE_TIMEOUT,
                false,
            )
            .await
            .expect("failed to write message to stream"),

            WriteCommand::Flush => writer.flush().await.expect("failed to flush write stream"),
        }
    }
}

#[instrument(skip(send_queue))]
async fn process_message<Q: AsRef<Sender<WriteCommand>>>(send_queue: Q, message: Message) {
    match message {
        Message::Ping => send_queue
            .as_ref()
            .send(WriteCommand::Send(Message::Pong))
            .await
            .expect("failed to send ping"),

        Message::PrepareStore { hash: _ } => {}

        message => error!("Unexpected message, cannot cope: {message:?}"),
    }
}
