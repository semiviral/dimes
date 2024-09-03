use super::{get_db, Error, Result};
use lib::chunk::Chunk;
use redb::TableDefinition;
use uuid::Uuid;

const CHUNK_PART_SIZE: usize = 100_000;

static CHUNK_TABLE: TableDefinition<([u8; size_of::<Uuid>()], u32), &[u8; CHUNK_PART_SIZE]> =
    TableDefinition::new("chunks");

pub fn get_chunk(id: &Uuid) -> Result<Option<Chunk>> {
    // TODO: use pooling for the chunk data
    let mut chunk = Chunk::new_zeroed(*id);
    let id_bytes = chunk.id().to_bytes_le();

    let read_txn = super::get_db().begin_read()?;
    let chunks_tbl = read_txn.open_table(CHUNK_TABLE)?;

    for (part_index, empty_part) in chunk.chunks_exact_mut(CHUNK_PART_SIZE).enumerate() {
        match chunks_tbl.get((id_bytes, part_index.try_into().unwrap()))? {
            Some(chunk_part) => {
                empty_part.copy_from_slice(chunk_part.value());
            }

            None if part_index == 0 => return Err(Error::KeyNotExists),
            None => return Err(Error::ChunkIncomplete),
        }
    }

    Ok(Some(chunk))
}

pub fn put_chunk(chunk: Chunk) -> Result<()> {
    let id_bytes = chunk.id().to_bytes_le();

    let write_txn = get_db().begin_write()?;

    let mut chunk_tbl = write_txn.open_table(CHUNK_TABLE).unwrap();
    for (part_index, chunk_part) in chunk.chunks_exact(CHUNK_PART_SIZE).enumerate() {
        let chunk_part = <&[u8; CHUNK_PART_SIZE]>::try_from(chunk_part).unwrap();
        chunk_tbl
            .insert((id_bytes, part_index.try_into().unwrap()), chunk_part)
            .unwrap();
    }
    drop(chunk_tbl);

    write_txn.commit()?;

    Ok(())
}
