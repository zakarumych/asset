
#[cfg(feature="futures")]
use futures::IntoFuture;

#[cfg(all(feature="ron", feature="serde"))]
use ron;

#[cfg(feature="serde")]
use serde::de::{DeserializeOwned};

#[cfg(feature="gfx-render")]
use hal::Backend;

#[cfg(feature="gfx-render")]
use render::Factory;

use std::io::Read;

pub trait AssetLoaderKind {
    const KIND: &'static str;
}

/// `AssetLoader` loads assets from raw data.
/// Some loaders can support several asset types and data formats. Such loaders implement `AssetLoader` for all supported asset-format pairs.
/// 
/// # Parameters
/// 
/// `A` - asset type produced by loader.
/// `F` - format type. Holds additional information required to decode asset from load data.
/// 
pub trait AssetLoader<A, F>: AssetLoaderKind {
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
pub trait Asset: Send + Sync + Sized + 'static {
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

#[cfg(feature="gfx-render")]
impl<B> AssetLoaderKind for Factory<B>
where
    B: Backend,
{
    const KIND: &'static str = "Factory";
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
impl AssetLoaderKind for SerdeLoader {
    const KIND: &'static str = "SerdeLoader";
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
        debug!("Loading asset with serde");
        format.from_reader(reader)
    }
}

#[cfg(all(feature="ron", feature="serde"))]
pub struct RonFormat;

#[cfg(all(feature="ron", feature="serde"))]
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
