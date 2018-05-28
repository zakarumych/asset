#[cfg(feature="obj")]
mod obj;

#[cfg(feature="obj")]
pub use self::obj::*;

use hal::Backend;
use render::Factory;

// pub use mesh::{Mesh, MeshBuilder}; // `cargo doc` panics because of this line: https://github.com/rust-lang/rust/issues/49883
use gfx_mesh::Mesh;

use asset::Asset;

impl<B> Asset for Mesh<B>
where
    B: Backend,
{
    type Loader = Factory<B>;

    const KIND: &'static str = "Mesh";
}
