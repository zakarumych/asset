use std::error::Error;
use std::io::Read;

#[cfg(feature="futures")]
use futures::{Stream, Future};

pub trait Asset<F, T>: Sized {
    /// Possible error.
    type Error: Error;

    /// Load asset from raw data.
    fn load<R>(read: R, format: F, aux: &mut T) -> Result<Self, Self::Error>
    where
        R: Read;
}

#[cfg(feature="futures")]
pub trait AsyncAsset<T>: Sized {
    type Future: Future<Item=Self>;

    fn load_async<S, R>(stream: S, aux: T) -> Self::Future
    where
        S: Stream<Item=R>,
        R: Read;
}

