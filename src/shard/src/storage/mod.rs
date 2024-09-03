pub mod chunk;
pub mod info;

use once_cell::sync::OnceCell;
use redb::{Database, Key, ReadOnlyTable, Table, TableDefinition, Value};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Transaction(#[from] redb::TransactionError),

    #[error(transparent)]
    Commit(#[from] redb::CommitError),

    #[error(transparent)]
    Table(#[from] redb::TableError),

    #[error(transparent)]
    Storage(#[from] redb::StorageError),

    #[error("specified key does not exist in database")]
    KeyNotExists,

    #[error("the specified chunk is missing parts")]
    ChunkIncomplete,
}

type Result<T> = std::result::Result<T, Error>;

static DATABASE: OnceCell<Database> = OnceCell::new();

pub fn init() {
    assert!(
        DATABASE.get().is_none(),
        "database has already been initialized"
    );

    let storage = crate::cfg::get().storage();
    let db = Database::create(storage.path()).expect("failed to open or create database");
    DATABASE
        .set(db)
        .expect("failed to set global databse reference");
}

fn get_db() -> &'static Database {
    DATABASE.get().expect("database has not been initialized")
}

fn with_table<T, K: Key, V: Value>(
    def: TableDefinition<K, V>,
    func: impl Fn(ReadOnlyTable<K, V>) -> T,
) -> Result<T> {
    let read_txn = get_db().begin_read()?;
    let table_ro = read_txn.open_table(def)?;

    Ok(func(table_ro))
}

fn with_table_mut<T, K: Key, V: Value>(
    def: TableDefinition<K, V>,
    mut func: impl FnMut(Table<K, V>) -> T,
) -> Result<T> {
    let write_txn = get_db().begin_write()?;
    let table_rw = write_txn.open_table(def)?;

    let result = func(table_rw);

    write_txn.commit()?;

    Ok(result)
}
