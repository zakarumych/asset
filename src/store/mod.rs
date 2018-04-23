#[cfg(feature="futures")]
mod async;

#[cfg(feature="fs")]
mod fs;

#[cfg(feature="futures")]
pub use self::async::{AsyncStore, AsyncStoreWrapper};

#[cfg(feature="fs")]
pub use self::fs::FsStore;

use std::io::Read;

#[cfg(feature="futures")]
pub use self::async::{AsyncStore, AsyncStoreWrapper};

/// Virtual container for assets.
/// Store can be represented by filesystem, archive, content server and so on.
/// 
/// # Parameters
/// `I` - identifier type the `Store` uses to identify assets.
/// 
pub trait Store<I: ?Sized> {
    /// Possible error type.
    type Error;

    /// Raw data reader.
    type Reader: Read;

    /// Get store kind.
    const KIND: &'static str;

    /// Fetch asset data from the store.
    /// Returns reader object that yields raw data of the asset.
    fn fetch(&mut self, id: &I) -> Result<Self::Reader, Self::Error>;
}


