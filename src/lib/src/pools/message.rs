use anyhow::Result;
use deadpool::managed::{Manager, Metrics, RecycleResult};
use deadpool::managed::{Object, Pool};
use once_cell::sync::OnceCell;

pub type ManagedMessage = Object<MessageBufManager>;

static MESSAGE_BUF_POOL: OnceCell<Pool<MessageBufManager>> = OnceCell::new();

pub fn configure_message_buf_pool(size: usize) -> Result<()> {
    MESSAGE_BUF_POOL
        .set(Pool::builder(MessageBufManager).max_size(size).build()?)
        .map_err(|_| anyhow!("pool already configured"))?;

    Ok(())
}

#[inline]
pub async fn get_message_buf() -> ManagedMessage {
    MESSAGE_BUF_POOL
        .get()
        .unwrap()
        .get()
        .await
        .expect("failed to get message buffer")
}

#[derive(Debug)]
pub struct MessageBufManager;

impl Manager for MessageBufManager {
    type Type = Vec<u8>;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(Vec::new())
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        obj.clear();

        Ok(())
    }
}
