use axum::{extract::Query, http::StatusCode, response::Response, routing::post, Json, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new().route("/resumable", post(resumable))
}

#[derive(Debug, Deserialize)]
struct Params {
    size: u64,
}

// TODO generalize this
#[derive(Debug, Deserialize)]
struct RequestMetadata {
    media_name: String,
}

async fn resumable(
    query: Query<Params>,
    metadata: Json<RequestMetadata>,
) -> (StatusCode, Response) {
    todo!()
}
