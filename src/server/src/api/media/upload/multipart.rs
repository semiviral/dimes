use axum::{extract::Multipart, http::StatusCode, routing::post, Router};

pub fn routes() -> Router {
    Router::new().route("/multipart", post(multipart))
}

async fn multipart(mut multipart: Multipart) -> StatusCode {
    
}
