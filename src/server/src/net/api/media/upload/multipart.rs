use axum::{extract::Multipart, http::StatusCode, response::Response, routing::post, Router};

pub fn routes() -> Router {
    Router::new().route("/multipart", post(multipart))
}

async fn multipart(mut multipart: Multipart) -> (StatusCode, Response) {
    todo!()
}
