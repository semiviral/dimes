use once_cell::sync::Lazy;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub server: Server,
    pub storage: Storage,
    pub caching: Caching,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub address: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    pub url: String,
    pub connections: u8,
    pub chunks: u32,
}

#[derive(Debug, Deserialize)]
pub struct Caching {
    pub chunks: usize,
    pub queues: u16,
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
