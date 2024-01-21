use crate::cfg;
use anyhow::Result;
use once_cell::sync::{Lazy, OnceCell};
use redis::{aio::MultiplexedConnection, Client};

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::open(cfg::get().storage.url.as_str()).expect("failed to open Redis client")
});
static CONNECTION: OnceCell<MultiplexedConnection> = OnceCell::new();

pub async fn connect() -> Result<()> {
    if let None = CONNECTION.get() {
        let connection = CLIENT.get_multiplexed_async_connection().await?;

        CONNECTION.set(connection).unwrap();
    } else {
        bail!("connection to Redis DB already established")
    }

    Ok(())
}

pub async fn with_connection<T>(with_fn: impl FnOnce(MultiplexedConnection) -> T) -> T {
    with_fn(
        CONNECTION
            .get()
            .cloned()
            .expect("connection to Redis DB has not been established"),
    )
}
