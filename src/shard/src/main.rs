#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod api;
mod cfg;

use anyhow::Result;
use lib::{api::shard, ConnectInfo, Token};
use once_cell::sync::{Lazy, OnceCell};

fn agent() -> String {
    format!("dimese-shard/{}", env!("CARGO_PKG_VERSION"))
}

fn info() -> (ConnectInfo, shard::Info) {
    (
        ConnectInfo { agent: agent() },
        shard::Info {
            max_chunks: 128,
            endpoint: String::new(),
        },
    )
}

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .use_native_tls()
        .build()
        .expect("failed to initialize HTTP client")
});

static TOKEN: OnceCell<Token> = OnceCell::new();

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    tokio::try_join!(api::start(), announce()).unwrap();

    std::process::exit(0)
}

async fn announce() -> Result<()> {
    let response = HTTP_CLIENT
        .post("http://127.0.0.1:3089/shards/register")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&info())?)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::CREATED => {
            let body = response.bytes().await?;
            let connect_info = serde_json::from_slice::<ConnectInfo>(&body)?;

            info!("Server connection info:\n{connect_info:#?}");

            Ok(())
        }

        status => {
            bail!("Register request failed with status code: {:?}", status)
        }
    }
}
