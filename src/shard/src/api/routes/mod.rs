use axum::{body::Body, http::StatusCode, response::Response, routing::get, Router};

mod chunk;

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/info", get(info))
        .nest("/api", chunk::routes())
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn info() -> (StatusCode, Response) {
    (
        StatusCode::OK,
        Response::builder()
            .header("Content-Type", "application/json")
            .header("Date", chrono::Utc::now().to_rfc3339())
            .body(Body::new(serde_json::to_string(&crate::info()).unwrap()))
            .unwrap(),
    )
}
