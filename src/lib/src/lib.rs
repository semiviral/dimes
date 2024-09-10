use uuid::Uuid;

#[macro_use]
extern crate tracing;

pub mod chunk;
// pub mod crypto;
// pub mod error;
pub mod array_pool;
pub mod net;
// pub mod buf;

pub const AGENT_STRING_MAX_LEN: usize = 32;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash([u8; 16]);

impl Hash {
    #[inline]
    pub const fn into_bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Debug)]
pub struct ShardInfo {
    id: Uuid,
    agent: String,
    chunks: i64,
}

impl ShardInfo {
    #[inline]
    pub fn new(id: Uuid, agent: String, chunks: u32) -> Self {
        Self {
            id,
            agent,
            chunks: chunks.into(),
        }
    }

    #[inline]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[inline]
    pub fn agent(&self) -> &str {
        self.agent.as_str()
    }

    #[inline]
    pub fn chunks(&self) -> i64 {
        self.chunks
    }
}

pub fn split_exact<const N: usize>(slice: &[u8]) -> Option<(&[u8; N], &[u8])> {
    (slice.len() >= N).then(|| {
        let slice_n = &slice[..N];
        let slice = &slice[N..];

        // SAFETY: Slice is split at `N`.
        (unsafe { slice_n.try_into().unwrap_unchecked() }, slice)
    })
}
