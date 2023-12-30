pub mod shards;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn accept_connections(listener: TcpListener, ctoken: &CancellationToken) -> Result<()> {
    let routes = Router::new().merge(shards::routes());

    axum::serve(listener, routes).await?;

    Ok(())
}
