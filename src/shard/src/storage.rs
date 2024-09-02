use anyhow::Result;
use lib::CHUNK_SIZE;
use redb::{Database, TableDefinition};
use tokio::sync::OnceCell;
use tracing::{Instrument, Level};
use uuid::Uuid;

use crate::cfg;

type UuidKey = [u8; 16];

const CHUNKS_TABLE_DEF: TableDefinition<UuidKey, [u8; CHUNK_SIZE]> = TableDefinition::new("chunks");

static CHUNKS_DB: OnceCell<Database> = OnceCell::const_new();

#[instrument]
pub async fn connect() -> Result<()> {
    

    debug!("Finished connecting to database.");

    Ok(())
}
