use directories::ProjectDirs;
use once_cell::sync::Lazy;
use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
    str::FromStr,
};
use uuid::Uuid;

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate sqlx;

mod api;
mod cfg;
mod storage;

static PROJECT_PATH: Lazy<ProjectDirs> = Lazy::new(|| {
    #[cfg(debug_assertions)]
    {
        ProjectDirs::from_path(PathBuf::from("./")).unwrap()
    }

    #[cfg(not(debug_assertions))]
    {
        ProjectDirs::from("net dimese", "", "dimese-shard").expect("`$HOME` directory required")
    }
});

static ID: Lazy<Uuid> = Lazy::new(|| {
    let id_path = PROJECT_PATH.data_dir().join("shard-id");
    let mut id_file = std::fs::File::options()
        .create(true)
        .read(true)
        .write(true)
        .open(id_path)
        .expect("error opening shard ID file");

    let mut buf = String::new();
    id_file
        .read_to_string(&mut buf)
        .expect("failed to read shard ID file contents");

    match Uuid::from_str(buf.as_str()) {
        Ok(id) => id,

        Err(_) => {
            id_file.rewind().unwrap();
            id_file.set_len(0).unwrap();

            let id = Uuid::now_v7();
            id_file
                .write_all(id.as_hyphenated().to_string().as_bytes())
                .expect("failed to write shard ID to file");

            id
        }
    }
});

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

async fn run() -> anyhow::Result<()> {
    storage::connect()
        .await
        .expect("failed to connect to storage database");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("failed to bind HTTP listener");
    api::accept_connections(listener).await?;

    Ok(())
}
