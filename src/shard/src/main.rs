use directories::ProjectDirs;
use once_cell::sync::Lazy;
use postgrest::Postgrest;
use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
    str::FromStr,
};
use uuid::Uuid;

#[macro_use]
extern crate tracing;

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

static ID: Lazy<Uuid> = Lazy::new(Uuid::new_v4);

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
