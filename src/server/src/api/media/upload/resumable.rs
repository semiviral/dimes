use axum::{extract::Query, http::StatusCode, response::Response, routing::post, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new().route("/resumable", post(resumable))
}

#[derive(Debug, Deserialize)]
struct Params {
    size: u64,
}

async fn resumable(query: Query<Params>) -> (StatusCode, Response) {
    todo!()
}
