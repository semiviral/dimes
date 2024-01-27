use anyhow::Result;
use lib::Hash;
use once_cell::sync::Lazy;
use redis::{aio::MultiplexedConnection, cmd, AsyncCommands, Client, Cmd, ConnectionLike};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Storage {
    _client: Client,
    connection: MultiplexedConnection,
}

impl Storage {
    pub async fn new<Str: AsRef<str>>(url: Str) -> Result<Self> {
        let client = Client::open(url.as_ref())?;
        let connection = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            _client: client,
            connection,
        })
    }

    pub async fn get_id(&mut self) -> Uuid {
        const SHARD_ID_KEY: &str = "SHARD_ID";

        static ID_GET_CMD: Lazy<Cmd> = Lazy::new(|| {
            let mut cmd = cmd("EXISTS");

            cmd.arg(SHARD_ID_KEY);

            cmd
        });

        static ID_SET_CMD: Lazy<Cmd> = Lazy::new(|| {
            let mut cmd = cmd("SET");

            cmd.arg(SHARD_ID_KEY)
                .arg(Uuid::now_v7().simple().to_string())
                .arg("NX");

            cmd
        });



        let result: String = self
            .connection
            .send_packed_command(&ID_GET_CMD)
            .await
            .unwrap();

        trace!("{:?}", result);

        self.connection
            .send_packed_command(&ID_GET_CMD)
            .await
            .unwrap();

        let (high_bits, low_bits) = self.connection.get(SHARD_ID_KEY).await.unwrap();
        Uuid::from_u64_pair(high_bits, low_bits)
    }

    #[instrument]
    pub async fn get_chunk_exists(&mut self, hash: Hash) -> bool {
        const CHUNK_HASHES_KEY: &str = "CHUNK_HASHES";

        self.connection
            .hexists(CHUNK_HASHES_KEY, &hash.into_bytes())
            .await
            .unwrap()
    }
}
