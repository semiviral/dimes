use anyhow::Result;
use axum::{extract::Query, http::HeaderMap, routing::put, Router};
use serde::Deserialize;
use tokio::net::TcpListener;

pub async fn accept_connections() -> Result<()> {
    let app = Router::new().route("/upload", put(upload_put));
    let listener = TcpListener::bind("127.0.0.1:3089").await?;
    debug!("Server is waiting to accept HTTP requests: {listener:?}");
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub enum UploadKind {
    Direct,
    Resumable,
}

impl std::str::FromStr for UploadKind {
    type Err = ();

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("direct") {
            Ok(Self::Direct)
        } else if s.eq_ignore_ascii_case("resumable") {
            Ok(Self::Resumable)
        } else {
            Err(())
        }
    }
}

async fn upload_put(headers: HeaderMap, Query(kind): Query<UploadKind>) {
    println!("{kind:?}");
}
