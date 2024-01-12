use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Cfg {
    pub server: Server,
    pub storage: Storage,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    pub url: String,
    pub chunks: u64,
}

pub fn get() -> &'static Cfg {
    use once_cell::sync::Lazy;

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
