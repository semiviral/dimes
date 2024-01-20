use anyhow::Result;
use deadpool::managed::{Manager, Metrics, RecycleResult};
use deadpool::managed::{Object, Pool};
use once_cell::sync::OnceCell;

pub type ManagedString = Object<StringManager>;

static STRING_BUF_POOL: OnceCell<Pool<StringManager>> = OnceCell::new();

pub fn configure_string_buf_pool(size: usize) -> Result<()> {
    STRING_BUF_POOL
        .set(Pool::builder(StringManager).max_size(size).build()?)
        .map_err(|_| anyhow!("pool already configured"))?;

    Ok(())
}

#[inline]
pub async fn get_string_buf() -> ManagedString {
    STRING_BUF_POOL
        .get()
        .unwrap()
        .get()
        .await
        .expect("failed to get message buffer")
}

#[derive(Debug)]
pub struct StringManager;

#[async_trait::async_trait]
impl Manager for StringManager {
    type Type = String;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(String::new())
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        obj.clear();

        Ok(())
    }
}
