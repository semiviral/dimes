use std::net::Ipv4Addr;

#[derive(Debug, clap::Parser)]
pub struct Arguments {
    #[arg(long = "server.ip")]
    pub bind_address: Ipv4Addr,

    #[arg(long = "server.port")]
    pub bind_port: u16,

    #[arg(long = "db.url")]
    pub db_url: String,

    #[arg(long = "db.max_chunks")]
    pub max_chunks: u64,
}
