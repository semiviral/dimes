use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
    routing::post,
    Json, Router,
};
use lib::net::types::ShardInfo;

use crate::api::responses;

pub fn routes() -> Router {
    Router::new().route("/register", post(register))
}

async fn register(headers: HeaderMap, shard_info: Json<ShardInfo>) -> (StatusCode, Response) {
    let na = &HeaderValue::try_from("n/a").unwrap();
    let agent = headers.get("user-agent").unwrap_or(na).to_str().unwrap();

    let db_store = crate::DB_STORE.read().await;
    let result = db_store
        .get()
        .unwrap()
        .add_shard(agent, shard_info.id(), shard_info.max_chunks())
        .await;

    // TODO check for max chunks count >0

    match result {
        Ok(_) => (StatusCode::OK, responses::empty()),
        Err(_) => (StatusCode::CONFLICT, responses::empty()),
    }
}
