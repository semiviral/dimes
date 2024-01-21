use crate::storage::with_connection;
use once_cell::sync::OnceCell;
use redis::AsyncCommands;
use uuid::Uuid;

static ID: OnceCell<Uuid> = OnceCell::new();

pub async fn get() -> Uuid {
    if let Some(id) = ID.get().cloned() {
        return id;
    }

    with_connection(|mut connection| async move {
        const SHARD_ID_KEY: &str = "SHARD_ID";

        match connection.get(SHARD_ID_KEY).await {
            Ok((high, low)) => Uuid::from_u64_pair(high, low),

            Err(_) => {
                let id = Uuid::now_v7();

                let _: () = connection
                    .set(SHARD_ID_KEY, id.as_u64_pair())
                    .await
                    .expect("failed to commit ID to Redis DB");

                id
            }
        }
    })
    .await
}
