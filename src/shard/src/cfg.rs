use once_cell::sync::Lazy;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct Cfg {
    tls: bool,
    remote: String,
    storage: Storage,
    message_timeout: u64,
}

impl Cfg {
    pub fn remote(&self) -> &str {
        self.remote.as_str()
    }

    pub fn use_tls(&self) -> bool {
        self.tls
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn message_timeout(&self) -> Duration {
        Duration::from_millis(self.message_timeout)
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
