use futures::io::{AllowStdIo, AsyncRead};

use store::Store;

/// Most stores are asynchronous in nature.
/// Therefore synchronous access to assets data is internally synchronized.
/// `futures` feature allows to work with asynchronous stores more naturally.
pub trait AsyncStore<I>: Store<I> {
    /// Stream of raw data chunks.
    type AsyncReader: AsyncRead;

    /// Fetch asset data from the store.
    /// Returns stream that yields raw data readers for the asset.
    fn fetch_async(&mut self, id: &I) -> Result<Self::AsyncReader, Self::Error>;
}

/// Wrapper for `Store` implementation which implements `AsyncStore` trivially
pub struct AsyncStoreWrapper<S> {
    store: S,
}

impl<I, S> Store<I> for AsyncStoreWrapper<S>
where
    I: ?Sized,
    S: Store<I>,
{
    type Error = S::Error;
    type Reader = S::Reader;

    const KIND: &'static str = S::KIND;

    fn fetch(&mut self, id: &I) -> Result<S::Reader, S::Error> {
        self.store.fetch(id)
    }
}

impl<I, S> AsyncStore<I> for AsyncStoreWrapper<S>
where
    S: Store<I>,
{
    type AsyncReader = AllowStdIo<S::Reader>;

    fn fetch_async(&mut self, id: &I) -> Result<AllowStdIo<S::Reader>, S::Error> {
        self.store.fetch(id).map(AllowStdIo::new)
    }
}
