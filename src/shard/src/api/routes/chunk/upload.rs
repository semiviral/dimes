use axum::{http::StatusCode, response::Response, routing::post, Router, Json};
use lib::api::shard::chunk::Upload;

pub fn routes() -> Router {
    Router::new().route("/upload", post(upload))
}

async fn upload(body: Json<Upload>) -> (StatusCode, Response) {
    todo!()
}
