use super::{Result, with_table, with_table_mut};
use chrono::{DateTime, Utc};
use redb::{AccessGuard, ReadOnlyTable, ReadableTable, Table, TableDefinition};
use uuid::Uuid;

static INFO_TABLE: TableDefinition<&str, &str> = TableDefinition::new("info");

struct InfoKey;

impl InfoKey {
    const SHARD_ID: &'static str = "shard_id";
    const STARTED_AT: &'static str = "started_at";
}

pub fn init() -> Result<()> {
    fn init_info_inner(mut info_tbl: Table<&str, &str>) -> Result<()> {
        // Set `shard_id` (if it has not been set)
        if info_tbl.get(InfoKey::SHARD_ID)?.is_none() {
            let new_shard_id_string = Uuid::now_v7().to_string();
            info_tbl.insert(InfoKey::SHARD_ID, new_shard_id_string.as_str())?;
        }

        // Set `started_at`
        let utc_now_string = Utc::now().to_rfc3339();
        info_tbl.insert(InfoKey::STARTED_AT, utc_now_string.as_str())?;

        Ok(())
    }

    with_table_mut(INFO_TABLE, init_info_inner)??;

    Ok(())
}

fn get_info<'a, 'b>(info_tbl: &ReadOnlyTable<&str, &str>, key: &str) -> AccessGuard<'a, &'b str> {
    info_tbl
        .get(key)
        .expect("failed to read table")
        .expect("info unavailable")
}

pub fn get_id() -> Uuid {
    with_table(INFO_TABLE, |info_tbl| {
        let shard_id_str = get_info(&info_tbl, InfoKey::SHARD_ID);
        let shard_id = Uuid::parse_str(shard_id_str.value()).expect("shard id is malformed");

        shard_id
    })
    .expect("failed to access database")
}

pub fn get_started_at() -> DateTime<Utc> {
    with_table(INFO_TABLE, |info_tbl| {
        let started_at_str = get_info(&info_tbl, InfoKey::STARTED_AT);
        let started_at = DateTime::parse_from_rfc3339(started_at_str.value())
            .expect("malformed input")
            .to_utc();

        started_at
    })
    .expect("failed to access database")
}
