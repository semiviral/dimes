use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::{patch, put},
    Router,
};
use serde::Deserialize;

pub fn add_routes(router: Router) -> Router {
    router.route("/upload", put(upload_put))
}

#[derive(Debug, Deserialize)]
pub struct UploadPut {
    kind: UploadKind,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadKind {
    Direct,
    Resumable,
}

async fn upload_put(headers: HeaderMap, kind: Query<UploadPut>) -> StatusCode {
    StatusCode::OK
}
