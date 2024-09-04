use anyhow::Result;
use deadpool::managed::{Manager, Metrics, Object, RecycleResult};

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

pub struct ManagedString(Object<StringManager>);

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
