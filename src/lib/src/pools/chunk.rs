use anyhow::Result;
use deadpool::managed::{Manager, Metrics, RecycleResult};
use deadpool::managed::{Object, Pool};
use once_cell::sync::OnceCell;
use std::alloc::Layout;

pub type ManagedChunk = Object<ChunkBufManager>;

static CHUNK_BUF_POOL: OnceCell<Pool<ChunkBufManager>> = OnceCell::new();

pub fn configure_chunk_buf_pool(size: usize) -> Result<()> {
    CHUNK_BUF_POOL
        .set(Pool::builder(ChunkBufManager).max_size(size).build()?)
        .map_err(|_| anyhow!("pool already configured"))?;

    Ok(())
}

#[inline]
pub async fn get_chunk_buf() -> ManagedChunk {
    CHUNK_BUF_POOL
        .get()
        .unwrap()
        .get()
        .await
        .expect("failed to get chunk buffer")
}

#[derive(Debug)]
pub struct ChunkBufManager;

impl Manager for ChunkBufManager {
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
