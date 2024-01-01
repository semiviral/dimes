mod info;
mod media;
pub mod response;

use anyhow::Result;
use axum::{ Router};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn accept_connections(listener: TcpListener, ctoken: &CancellationToken) -> Result<()> {
    let router = Router::new()
        .nest("/api", info::routes())
        .nest("/api", media::routes());

    axum::serve(listener, router).await?;

    Ok(())
}
