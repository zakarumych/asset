#[cfg(feature = "png")]
mod png;

#[cfg(feature = "png")]
pub use self::png::*;

use hal::Backend;
use render::Factory;

use asset::Asset;
use gfx_texture::Texture;

impl<B> Asset for Texture<B>
where
    B: Backend,
{
    type Loader = Factory<B>;

    const KIND: &'static str = "Texture";
}
