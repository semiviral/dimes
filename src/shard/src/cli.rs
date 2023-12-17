use std::net::Ipv4Addr;

#[derive(Debug, clap::Parser)]
pub struct Arguments {
    #[arg(long = "server.ip", default_value = "127.0.0.1")]
    pub bind_address: Ipv4Addr,

    #[arg(long = "server.port", default_value = "3088")]
    pub bind_port: u16,

    #[arg(long = "db.url", default_value = "redis://127.0.0.1:6379")]
    pub db_url: String,

    #[arg(long = "db.max_chunks", default_value = "128")]
    pub max_chunks: u64,
}
