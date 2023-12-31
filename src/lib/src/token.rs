use serde::{Deserialize, Serialize};
use std::{fmt, marker::PhantomData};
use uuid::Uuid;

pub trait Kind {}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Server;
impl Kind for Server {}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Shard;
impl Kind for Shard {}

#[repr(transparent)]
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token<K: Kind>(Uuid, PhantomData<K>);

impl<K: Kind> Token<K> {
    #[inline]
    pub fn generate() -> Self {
        Self(Uuid::now_v7(), PhantomData)
    }
}

impl<K: Kind> fmt::Debug for Token<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Token").field(&self.0).finish()
    }
}

impl<K: Kind> fmt::Display for Token<K> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
