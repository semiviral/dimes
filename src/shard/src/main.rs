#[macro_use]
extern crate tracing;
#[macro_use]
extern crate anyhow;

mod api;
mod cfg;

use std::future::IntoFuture;

use lib::{
    api::{response_json, shard},
    token::{Server, Shard, Token},
    ConnectInfo,
};
use once_cell::sync::{Lazy, OnceCell};
use tokio_util::sync::CancellationToken;

static SHARD_TOKEN: Lazy<Token<Shard>> = Lazy::new(Token::generate);
static SERVER_TOKEN: OnceCell<Token<Server>> = OnceCell::new();

fn agent() -> String {
    format!("dimese-shard/{}", env!("CARGO_PKG_VERSION"))
}

fn info() -> (ConnectInfo<Shard>, shard::Info) {
    (
        ConnectInfo {
            agent: agent(),
            token: *SHARD_TOKEN,
        },
        shard::Info {
            max_chunks: 128,
            endpoint: String::new(),
        },
    )
}

static CTOKEN: Lazy<CancellationToken> = Lazy::new(CancellationToken::new);

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .use_native_tls()
        .build()
        .expect("failed to initialize HTTP client")
});

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let api_join = tokio::spawn(api::start());

    let response = HTTP_CLIENT
        .post("http://127.0.0.1:3089/api/shard/register")
        .json(&info())
        .send()
        .await
        .expect("request failed");

    let connect_info = response_json::<ConnectInfo<Server>>(response)
        .await
        .expect("unexpected response");
    info!("Server connection info:\n{connect_info:#?}");
    SERVER_TOKEN
        .set(connect_info.token)
        .expect("token has already been set (this is unexpected).");

    tokio::select! {
        _ = CTOKEN.cancelled() => {}

        result = api_join.into_future() => {
            result.expect("join error").expect("api error");
        }
    }
}
