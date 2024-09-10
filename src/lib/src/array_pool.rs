use deadpool::managed::{Manager, Object, Pool, QueueMode};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Pool,
}

pub struct ArrayPool<const N: usize>(Pool<ArrayManager<N>>);

impl<const N: usize> ArrayPool<N> {
    pub fn new(max_size: usize) -> Self {
        let pool = Pool::builder(ArrayManager)
            .max_size(max_size)
            .queue_mode(QueueMode::Lifo)
            .build()
            .unwrap();

        Self(pool)
    }

    pub async fn get(&self) -> Result<ManagedArray<N>> {
        self.0
            .get()
            .await
            .map(ManagedArray)
            .map_err(|_| Error::Pool)
    }
}

#[derive(Debug)]
struct ArrayManager<const N: usize>;

impl<const N: usize> Manager for ArrayManager<N> {
    type Type = Box<[u8; N]>;
    type Error = std::convert::Infallible;

    async fn create(&self) -> std::result::Result<Self::Type, Self::Error> {
        use std::alloc::Layout;

        // SAFETY: Allocate a zero-initialized `LEN`-sized region of memory, and wrap it in a box.
        unsafe {
            let memory = std::alloc::alloc_zeroed(Layout::new::<[u8; N]>());
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(memory, N);
            let boxed_slice = Box::from_raw(slice_ptr);
            let boxed_array = boxed_slice.try_into().unwrap_unchecked();

            Ok(boxed_array)
        }
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        obj.fill(0);

        Ok(())
    }
}

#[derive(Debug)]
pub struct ManagedArray<const N: usize>(Object<ArrayManager<N>>);

impl<const N: usize> std::ops::Deref for ManagedArray<N> {
    type Target = Box<[u8; N]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> std::ops::DerefMut for ManagedArray<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
