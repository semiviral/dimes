use crate::net::api;
use axum::{http::StatusCode, response::Response, routing::get, Router};
use once_cell::sync::Lazy;
use serde::Serialize;

pub fn routes() -> Router {
    Router::new().route("/info", get(info))
}

static INFO: Lazy<Info> = Lazy::new(|| Info {
    agent: crate::agent(),
});

// TODO make this type general and / or public
#[derive(Debug, Serialize)]
struct Info {
    agent: String,
}

async fn info() -> (StatusCode, Response) {
    (StatusCode::OK, api::response::json(&*INFO).unwrap())
}
