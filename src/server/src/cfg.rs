use once_cell::sync::Lazy;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub bind: Bind,
    //pub db: Db,
}

#[derive(Debug, Deserialize)]
pub struct Bind {
    pub shard: SocketAddr,
    pub http: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct Db {
    pub url: String,
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
