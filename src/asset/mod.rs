
#[cfg(feature="futures")]
use futures::IntoFuture;

use std::io::Read;

/// `AssetLoader` loads assets from raw data.
/// Some loaders can support several asset types and data formats. Such loaders implement `AssetLoader` for all supported asset-format pairs.
/// 
/// # Parameters
/// 
/// `A` - asset type produced by loader.
/// `F` - format type. Holds additional information required to decode asset from load data.
/// 
pub trait AssetLoader<A, F> {
    /// Possible error type.
    type Error;

    /// Load asset from raw data.
    fn load<R>(&mut self, format: F, read: R) -> Result<A, Self::Error>
    where
        R: Read;
}

#[cfg(feature="futures")]
/// `AssetStreamingLoader` can load assets from data chunks.
pub trait AsyncAssetLoader<A, F, R>: AssetLoader<A, F> + Sized {
    type Loader: IntoFuture<Item = (Self, A), Error = (Self, Self::Error)>;
    fn load_async(self, format: F, reader: R) -> Self::Loader;
}


