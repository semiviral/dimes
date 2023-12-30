use anyhow::Result;
use tokio::net::TcpListener;

mod routes;

pub async fn start() -> Result<()> {
    let listener = TcpListener::bind(crate::cfg::get().bind).await?;

    axum::serve(listener, routes::router()).await?;

    Ok(())
}
