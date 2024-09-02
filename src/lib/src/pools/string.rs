use anyhow::Result;
use deadpool::managed::{Manager, Metrics, RecycleResult};
use deadpool::managed::{Object, Pool};
use once_cell::sync::{Lazy, OnceCell};
use serde::de::Visitor;
use tokio::runtime::Runtime;

pub struct ManagedString(Object<StringManager>);

impl serde::Serialize for ManagedString {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        (*self.0).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ManagedString {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_str(ManagedStringVisitor::get())
    }
}

impl std::fmt::Debug for ManagedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ManagedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Deref for ManagedString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ManagedString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct ManagedStringVisitor(ManagedString);

impl ManagedStringVisitor {
    fn get() -> Self {
        static STR_BUF_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        });

        Self(STR_BUF_RUNTIME.block_on(get_string_buf()))
    }
}

impl<'de> Visitor<'de> for ManagedStringVisitor {
    type Value = ManagedString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a valid UTF-8 string")
    }

    fn visit_str<E>(mut self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0 .0.push_str(v);

        Ok(self.0)
    }
}

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
        .map(ManagedString)
        .expect("failed to get message buffer")
}

#[derive(Debug)]
pub struct StringManager;

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
