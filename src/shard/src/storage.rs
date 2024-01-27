use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing::{Instrument, Level};

use crate::cfg;

#[instrument]
pub async fn connect() -> Result<()> {
    static MIGRATOR: sqlx::migrate::Migrator = migrate!();

    let connect_str = cfg::get().storage.url.as_str();
    event!(Level::DEBUG, storage = connect_str);

    let pool = PgPoolOptions::new()
        .min_connections(cfg::get().storage.connections.min)
        .max_connections(cfg::get().storage.connections.max)
        .connect(connect_str)
        .await
        .expect("failed to connect to database");

    MIGRATOR
        .run(&pool)
        .instrument(span!(Level::TRACE, "migrations"))
        .await?;

    debug!("Finished connecting to database.");

    Ok(())
}
