use once_cell::sync::Lazy;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Cfg {
    bind: SocketAddr,
    storage: Storage,
}

impl Cfg {
    pub fn bind(&self) -> &SocketAddr {
        &self.bind
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    path: String,
    chunks: u64,
}

impl Storage {
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn chunks(&self) -> u64 {
        self.chunks
    }
}

pub fn get() -> &'static Cfg {
    static APP_CONFIG: Lazy<Cfg> = Lazy::new(|| {
        use config::{Config, Environment};

        Config::builder()
            .add_source(
                Environment::with_prefix("DIMESE_SHARD")
                    .separator("_")
                    .list_separator(","),
            )
            .build()
            .and_then(Config::try_deserialize)
            .expect("failed to read configuration")
    });

    &APP_CONFIG
}
