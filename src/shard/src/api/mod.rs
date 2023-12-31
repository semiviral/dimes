use anyhow::Result;
use tokio::net::TcpListener;

mod routes;

pub async fn start() -> Result<()> {
    axum::serve(
        TcpListener::bind(crate::cfg::get().bind).await?,
        routes::router(),
    )
    .await?;

    Ok(())
}
