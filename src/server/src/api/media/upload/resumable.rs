use axum::{extract::Query, http::StatusCode, response::Response, routing::post, Router, Json};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new().route("/resumable", post(resumable))
}

#[derive(Debug, Deserialize)]
struct Params {
    size: u64,
}

#[derive(Debug, Deserialize)]
struct RequestMetadata {
    media_name: String,
}

#[derive(Debug, Deserialize)]
struct VideoMetadata {
    format: String // TODO create format type
}

async fn resumable(query: Query<Params>, metadata: Json<RequestMetadata>) -> (StatusCode, Response) {
    
}
