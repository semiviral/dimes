use once_cell::sync::Lazy;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub bind: Bind,
    pub storage: Storage,
}

#[derive(Debug, Deserialize)]
pub struct Bind {
    pub http: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    pub url: String,
    pub chunks: u64,
    pub connections: Connections,
}

#[derive(Debug, Deserialize)]
pub struct Connections {
    pub min: u32,
    pub max: u32,
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
