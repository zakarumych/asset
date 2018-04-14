
#[cfg(feature="futures")]
use futures::IntoFuture;

#[cfg(all(feature="ron", feature="serde"))]
use ron;

#[cfg(feature="serde")]
use serde::de::{DeserializeOwned};

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
    fn load<R>(&mut self, format: F, reader: R) -> Result<A, Self::Error>
    where
        R: Read;
}

#[cfg(feature="futures")]
/// `AssetStreamingLoader` can load assets from data chunks.
pub trait AsyncAssetLoader<A, F, R>: AssetLoader<A, F> + Sized {
    type Loader: IntoFuture<Item = (Self, A), Error = (Self, Self::Error)>;
    fn load_async(self, format: F, reader: R) -> Self::Loader;
}

/// Asset type specifies loader type.
pub trait Asset: Sized {
    /// Loader type for the asset.
    type Loader;

    const KIND: &'static str;

    /// Load asset using loader.
    fn load<F, R>(loader: &mut Self::Loader, format: F, reader: R) -> Result<Self, <Self::Loader as AssetLoader<Self, F>>::Error>
    where
        R: Read,
        Self::Loader: AssetLoader<Self, F>,
    {
        loader.load(format, reader)
    }
}


#[cfg(feature="serde")]
pub struct SerdeLoader;

#[cfg(feature="serde")]
pub trait SerdeFormat {
    type Error;

    fn from_reader<D, R>(self, reader: R) -> Result<D, Self::Error>
    where
        D: DeserializeOwned,
        R: Read;
}


#[cfg(feature="serde")]
impl<A, F> AssetLoader<A, F> for SerdeLoader
where
    A: DeserializeOwned,
    F: SerdeFormat,
{
    type Error = F::Error;

    fn load<R>(&mut self, format: F, reader: R) -> Result<A, Self::Error>
    where
        R: Read,
    {
        format.from_reader(reader)
    }
}

#[cfg(all(feature="ron", feature="serde"))]
pub struct RonFormat;

impl SerdeFormat for RonFormat {
    type Error = ron::de::Error;

    fn from_reader<D, R>(self, reader: R) -> Result<D, Self::Error>
    where
        D: DeserializeOwned,
        R: Read,
    {
        ron::de::from_reader(reader)
    }
}
