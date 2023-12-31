use axum::{http::StatusCode, response::Response, routing::post, Json, Router};
use chrono::Utc;
use lib::{
    api::shard,
    error_code::ErrorCode,
    token::{Server, Shard, Token},
    ChunkHash, ConnectInfo,
};
use std::collections::BTreeMap;
use tokio::sync::RwLock;
use tracing::Level;

static SHARD_MAP: RwLock<BTreeMap<Token<Shard>, Token<Server>>> =
    RwLock::const_new(BTreeMap::new());
static SHARD_STORAGE: BTreeMap<ChunkHash, Vec<Token<Shard>>> = BTreeMap::new();

pub fn routes() -> Router {
    Router::new().route("/shard/register", post(register))
}

async fn register(
    Json((connect_info, shard_info)): Json<(ConnectInfo<Shard>, shard::Info)>,
) -> (StatusCode, Response) {
    let span = span!(Level::TRACE, "shard", token = %connect_info.token);
    let _enter = span.enter();

    event!(Level::DEBUG, connect_info = ?connect_info, shard_info = ?shard_info);

    let shard_map = SHARD_MAP.read().await;
    if shard_map.contains_key(&connect_info.token) {
        trace!("Shard token already registered.");

        let error_json = serde_json::to_string(&ErrorCode::AlreadyRegistered).unwrap();

        (
            StatusCode::CONFLICT,
            Response::builder()
                .header("Date", Utc::now().to_rfc3339())
                .header("Content-Type", "application/json")
                .body(error_json.into())
                .unwrap(),
        )
    } else {
        trace!("Registering shard token...");

        let server_token = Token::<Server>::generate();

        drop(shard_map);
        let mut shard_map = SHARD_MAP.write().await;
        shard_map.insert(connect_info.token, server_token);
        drop(shard_map);

        trace!("Shard token registered.");

        let json = serde_json::to_string(&ConnectInfo {
            agent: crate::agent(),
            token: server_token,
        })
        .unwrap();

        let response = Response::builder()
            .header("Date", chrono::Utc::now().to_rfc3339())
            .header("Content-Type", "application/json")
            .body(json.into())
            .unwrap();

        (StatusCode::CREATED, response)
    }
}
