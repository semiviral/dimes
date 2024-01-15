use once_cell::sync::Lazy;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub server: Server,
    pub storage: Storage,
    pub pooling: Pooling,
    pub queuing: Queuing,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub address: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    pub url: String,
    pub chunks: usize,
}

#[derive(Debug, Deserialize)]
pub struct Pooling {
    pub chunks: usize,
}

#[derive(Debug, Deserialize)]
pub struct Queuing {
    pub send: usize,
}

pub fn get() -> &'static Cfg {
    static APP_CONFIG: Lazy<Cfg> = Lazy::new(|| {
        use config::{Config, Environment};

        Config::builder()
            .add_source(
                Environment::with_prefix("DIMESE")
                    .separator("_")
                    .list_separator(","),
            )
            .build()
            .and_then(Config::try_deserialize)
            .expect("failed to read configuration")
    });

    &APP_CONFIG
}
