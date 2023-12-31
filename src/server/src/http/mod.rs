pub mod shard;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn accept_connections(listener: TcpListener, ctoken: &CancellationToken) -> Result<()> {
    let routes = Router::new().nest("/api", shard::routes());

    tokio::select! {
        result = async { axum::serve(listener, routes).await } => {
            result?;

            Ok(())
        }

        _ = ctoken.cancelled() => { Ok(()) }
    }
}
