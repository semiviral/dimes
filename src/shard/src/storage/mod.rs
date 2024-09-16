pub mod chunk;
pub mod info;

use anyhow::Result;
use once_cell::sync::OnceCell;
use redb::{Database, Key, ReadOnlyTable, Table, TableDefinition, Value};

static DATABASE: OnceCell<Database> = OnceCell::new();

pub fn init() {
    let storage = crate::cfg::get().storage();
    let db = Database::create(storage.path()).expect("failed to open or create database");

    debug!("Verifying database...");
    let write_txn = db
        .begin_write()
        .expect("failed to open write to init tables");
    write_txn
        .open_table(info::TABLE_DEF)
        .expect("failed to initialize info table");
    write_txn
        .open_table(chunk::TABLE_DEF)
        .expect("failed to initialize chunk table");
    write_txn
        .commit()
        .expect("failed to commit table initialization");

    DATABASE.set(db).expect("database already initialized");
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
