use anyhow::Result;
use axum::{body::Body, http::response::Builder, response::Response};

pub fn default() -> Builder {
    Response::builder()
        .header("Date", chrono::Utc::now().to_rfc3339())
        .header("Server", crate::agent())
}

pub fn empty() -> Response {
    default().body(Body::empty()).unwrap()
}

pub fn json<T: serde::Serialize>(item: T) -> Result<Response> {
    let body = serde_json::to_string(&item)?;

    let resposne = default()
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len())
        .body(Body::from(body))?;

    Ok(resposne)
}
