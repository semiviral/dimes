use axum::{http::StatusCode, routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/health", get(|| async { StatusCode::OK }))
}
