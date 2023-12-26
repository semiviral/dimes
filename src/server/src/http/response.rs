use axum::response::Response;

fn default_response() -> axum::http::response::Builder {
    Response::builder().header("Date", chrono::Utc::now().to_rfc3339()).header("Server", crate::agent())
}

fn json_response<T: serde::Serialize>(item: T) -> Result<Response> {
    default_response().header("Content-Type", "text/json")
}