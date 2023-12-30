use axum::{body::Body, http::StatusCode, response::Response, routing::post, Json, Router};
use lib::{ConnectInfo, ShardInfo};

pub fn routes() -> Router {
    Router::new().route("/register", post(_post))
}

async fn _post(connect_info: Json<(ConnectInfo, ShardInfo)>) -> (StatusCode, Response) {
    println!("{connect_info:?}");

    let response = Response::builder()
        .header("Date", chrono::Utc::now().to_rfc3339())
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(&crate::info()).unwrap()))
        .unwrap();

    (StatusCode::CREATED, response)
}
