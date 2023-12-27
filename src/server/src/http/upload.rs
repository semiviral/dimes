use anyhow::Result;
use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{patch, post, put},
    Router,
};
use serde::Deserialize;

use super::response::default_response;

pub fn add_routes(router: Router) -> Router {
    router.route("/upload", post(_post))
}

#[derive(Debug, Deserialize)]
pub struct _Post {
    kind: Kind,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Direct,
    Resumable,
}

async fn _post(headers: HeaderMap, query: Query<_Post>) -> (StatusCode, Response) {
    match query.kind {
        Kind::Direct => {
            let Some(content_len) = headers
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
            else {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    default_response()
                        .body(Body::from(
                            "The request must provide a valid `Content-Length` header.",
                        ))
                        .unwrap(),
                );
            };

            let response = crate::http::response::default_response()
                .header("Location", "http://nothing.com/")
                .body(Body::empty())
                .unwrap();

            (StatusCode::CREATED, response)
        }
        Kind::Resumable => todo!(),
    }
}
