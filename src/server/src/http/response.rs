use anyhow::Result;
use axum::{body::Body, http::response::Builder, response::Response};

pub fn default_response() -> Builder {
    Response::builder()
        .header("Date", chrono::Utc::now().to_rfc3339())
        .header("Server", crate::agent())
}

pub fn json_response<T: serde::Serialize>(item: T) -> Result<Response> {
    let body = serde_json::to_string(&item)?;

    let builder = default_response()
        .header("Content-Length", body.len())
        .header("Content-Type", "application/json")
        .body(Body::from(body))?;

    Ok(builder)
}
