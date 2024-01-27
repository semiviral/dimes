use crate::cfg;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

pub fn routes() -> Router {
    Router::new().route("/info", get(info))
}

#[derive(Debug, Serialize)]
struct Info {
    pub chunks: u64,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            chunks: cfg::get().storage.chunks,
        }
    }
}

async fn info() -> impl IntoResponse {
    (StatusCode::OK, Json(Info::default()))
}
