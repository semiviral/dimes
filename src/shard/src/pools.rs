use anyhow::Result;
use axum::async_trait;
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use lib::net::CHUNK_SIZE;
use once_cell::sync::Lazy;
use std::alloc::Layout;

type Chunk = Box<[u8; CHUNK_SIZE]>;

pub struct ChunkManager;

#[async_trait]
impl Manager for ChunkManager {
    type Type = Chunk;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type> {
        const LAYOUT: Layout = Layout::new::<[u8; CHUNK_SIZE]>();

        // SAFETY: Ptr is immediately boxed.
        let chunk = unsafe {
            let ptr = std::alloc::alloc(LAYOUT);
            let size = LAYOUT.size();
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr, size).cast();

            Box::from_raw(slice_ptr)
        };

        Ok(chunk)
    }

    async fn recycle(&self, _: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

use crate::cfg;

static CHUNK_POOL: Lazy<Pool<ChunkManager>> = Lazy::new(|| {
    Pool::builder(ChunkManager)
        .max_size(cfg::get().pooling.chunks)
        .build()
        .unwrap()
});

#[inline]
pub async fn get_chunk() -> Object<ChunkManager> {
    CHUNK_POOL.get().await.unwrap()
}
