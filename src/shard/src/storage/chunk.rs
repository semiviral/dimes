use super::{get_db, Result};
use lib::chunk::Chunk;
use redb::TableDefinition;
use uuid::Uuid;

pub(super) static TABLE_DEF: TableDefinition<[u8; size_of::<Uuid>()], &[u8; Chunk::SIZE]> =
    TableDefinition::new("chunks");

pub fn chunk_exists(id: Uuid) -> Result<bool> {
    Ok(get_db()
        .begin_read()?
        .open_table(TABLE_DEF)?
        .get(id.to_bytes_le())?
        .is_some())
}

pub fn get_chunk(id: Uuid) -> Result<Option<Chunk>> {
    let id_bytes = id.to_bytes_le();

    let read_txn = get_db().begin_read()?;
    let chunk_tbl = read_txn.open_table(TABLE_DEF)?;

    let chunk = chunk_tbl.get(id_bytes)?.map(|stored_chunk| {
        // TODO: use pooling for the chunk data
        let mut chunk = Chunk::new_zeroed(id);
        chunk.copy_from_slice(stored_chunk.value());
        chunk
    });

    Ok(chunk)
}

pub fn put_chunk(chunk: Chunk) -> Result<()> {
    let write_txn = get_db().begin_write()?;

    write_txn
        .open_table(TABLE_DEF)?
        .insert(chunk.id().to_bytes_le(), &*chunk)?;

    write_txn.commit()?;

    Ok(())
}
