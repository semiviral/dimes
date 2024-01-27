use anyhow::Result;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use lib::net::{receive_message, send_message, Message, MAX_TIMEOUT, MESSAGE_TIMEOUT};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
};
use uuid::Uuid;

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod cfg;
// mod storage;
mod api;

fn agent_str() -> &'static str {
    concat!("dimese-shard/", env!("CARGO_PKG_VERSION"))
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
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    api::accept_connections(listener).await?;

    Ok(())

    //    trace!("Attempting to connect to server...");

    // let mut storage = storage::Storage::new(cfg::get().storage.url.as_str())
    //     .await
    //     .expect("failed to initialize storage");
    // let id = storage.get_id().await;

    // debug!("Shard ID is: {id}");

    // loop {}

    // let (stream, cipher) = connect_server(id).await?;

    // let (reader, writer) = stream.into_split();
    // let mut reader = BufReader::new(reader);
    // let writer = BufWriter::new(writer);

    // let (send_queue, recv_queue) = tokio::sync::mpsc::channel(cfg::get().caching.queues.into());

    // tokio::spawn(process_writes(writer, Arc::clone(&cipher), recv_queue));

    // loop {
    //     let message = receive_message(&mut reader, &cipher, MAX_TIMEOUT).await?;
    //     tokio::spawn(process_message(send_queue.clone(), message));
    // }
}

async fn connect_server(id: Uuid) -> Result<(TcpStream, Arc<XChaCha20Poly1305>)> {
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

    let info_message = {
        let mut agent = lib::pools::get_string_buf().await;
        agent.push_str(agent_str());

        Message::ShardInfo {
            id,
            agent,
            chunks: cfg::get().storage.chunks,
        }
    };

    // Send info block to server.
    send_message(&mut stream, &cipher, info_message, MESSAGE_TIMEOUT, true).await?;

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
async fn process_message(send_queue: Sender<WriteCommand>, message: Message) {
    process_message_impl(send_queue, message)
        .await
        .expect("error processing message");
}

async fn process_message_impl(send_queue: Sender<WriteCommand>, message: Message) -> Result<()> {
    match message {
        Message::Ping => {
            send_queue.send(WriteCommand::Send(Message::Pong)).await?;

            Ok(())
        }

        Message::Store { hash, chunk } => {
            send_queue.send(WriteCommand::Send(Message::Ok)).await?;

            Ok(())
        }

        message => bail!("Unexpected message, cannot cope: {message:?}"),
    }
}
