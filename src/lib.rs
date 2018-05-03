
#[macro_use]
extern crate derivative;

extern crate failure;

#[cfg(feature="futures")]
extern crate futures;

#[cfg(feature="gfx-hal")]
extern crate gfx_hal as hal;

#[cfg(feature="gfx-mesh")]
extern crate gfx_mesh;

#[cfg(feature="gfx-render")]
extern crate gfx_render as render;

#[cfg(feature="obj")]
extern crate obj;

#[cfg(feature="png")]
extern crate png;

#[cfg(any(test, feature="ron"))]
extern crate ron;

#[cfg(any(test, feature="serde"))]
#[macro_use]
extern crate serde;


#[cfg(feature="mesh")]
mod mesh;

#[cfg(feature="sprite")]
mod sprite;

#[cfg(any(feature="texture"))]
mod texture;

mod asset;
mod handle;
mod manager;
mod store;

pub use asset::*;
pub use handle::*;
pub use manager::*;
pub use store::*;

#[cfg(feature="mesh")]
pub use mesh::*;

#[cfg(feature="sprite")]
pub use sprite::*;

#[cfg(any(feature="texture"))]
pub use texture::*;

#[cfg(test)]
mod tests;
