pub mod upload;
pub mod response;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn accept_connections(listener: TcpListener, ctoken: &CancellationToken) -> Result<()> {
    let router = Router::new();
    let router = upload::add_routes(router);

    axum::serve(listener, router).await?;

    Ok(())
}
