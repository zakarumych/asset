
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

pub mod asset;
pub mod handle;
pub mod manager;
pub mod store;

#[cfg(feature="mesh")]
pub mod mesh;

#[cfg(feature="sprite")]
pub mod sprite;

#[cfg(any(feature="texture"))]
pub mod texture;

#[cfg(test)]
mod tests;
