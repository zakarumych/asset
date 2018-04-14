
#[cfg(feature="png")]
pub mod png;

use std::borrow::Cow;

use failure::Error;

use hal::Backend;
use hal::format::{Aspects, Format};
use hal::image::{Tiling, Kind, Usage, Access, StorageFlags, Layout, SubresourceLayers, Offset};
use hal::memory::{Pod, Properties, cast_slice};

use render::{Factory, Image};

use asset::Asset;

pub struct Texture<B: Backend> {
    kind: Kind,
    format: Format,
    image: Image<B>,
}

impl<B> Texture<B>
where
    B: Backend,
{
    pub fn new<'a>(kind: Kind) -> TextureBuilder<'a> {
        TextureBuilder::new(kind)
    }

    pub fn image(&self) -> &Image<B> {
        &self.image
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}

impl<B> Asset for Texture<B>
where
    B: Backend,
{
    type Loader = Factory<B>;

    const KIND: &'static str = "Texture";
}

pub struct TextureBuilder<'a> {
    kind: Kind,
    format: Format,
    data_width: u32,
    data_height: u32,
    data: Cow<'a, [u8]>,
}

impl<'a> TextureBuilder<'a> {
    pub fn new(kind: Kind) -> Self {
        let extent = kind.extent();
        TextureBuilder {
            kind: kind,
            format: Format::Rgba8Srgb,
            data_width: extent.width,
            data_height: extent.height,
            data: Vec::new().into(),
        }
    }

    pub fn with_format(mut self, format: Format) -> Self {
        self.set_format(format);
        self
    }

    pub fn set_format(&mut self, format: Format) -> &mut Self {
        assert_eq!(format.aspects(), Aspects::COLOR);
        self.format = format;
        self
    }

    pub fn with_data_width(mut self, data_width: u32) -> Self {
        self.set_data_width(data_width);
        self
    }

    pub fn set_data_width(&mut self, data_width: u32) -> &mut Self {
        assert!(data_width >= self.kind.extent().width);
        self.data_width = data_width;
        self
    }

    pub fn with_data_height(mut self, data_height: u32) -> Self {
        self.set_data_height(data_height);
        self
    }

    pub fn set_data_height(&mut self, data_height: u32) -> &mut Self {
        assert!(data_height >= self.kind.extent().height);
        self.data_height = data_height;
        self
    }

    pub fn with_data<D, P>(mut self, data: D) -> Self
    where
        D: Into<Cow<'a, [P]>>,
        P: Pod + 'a,
    {
        self.set_data(data);
        self
    }

    pub fn set_data<D, P>(&mut self, data: D) -> &mut Self
    where
        D: Into<Cow<'a, [P]>>,
        P: Pod + 'a,
    {
        self.data = cast_cow(data.into());
        self
    }

    pub fn build<B>(&self, factory: &mut Factory<B>) -> Result<Texture<B>, Error>
    where
        B: Backend,
    {
        let extent = self.kind.extent();
        assert!(self.data_width >= extent.width);
        assert!(self.data.len() * 8 >= (self.data_width * extent.height * extent.depth * self.format.base_format().0.desc().bits as u32) as usize);

        let mut image = factory.create_image(
            self.kind,
            1,
            self.format,
            Tiling::Optimal,
            Properties::DEVICE_LOCAL,
            Usage::TRANSFER_DST | Usage::SAMPLED,
            StorageFlags::empty(),
        )?;

        factory.upload_image(
            &mut image,
            Layout::ShaderReadOnlyOptimal,
            Access::SHADER_READ,
            SubresourceLayers {
                aspects: Aspects::COLOR,
                level: 0,
                layers: 0..1,
            },
            Offset::ZERO,
            self.kind.extent(),
            self.data_width,
            self.data_height,
            &self.data,
        )?;

        Ok(Texture {
            kind: self.kind,
            format: self.format,
            image,
        })
    }
}

fn cast_vec<A: Pod, B: Pod>(mut vec: Vec<A>) -> Vec<B> {
    use std::mem;

    let raw_len = mem::size_of::<A>() * vec.len();
    let len = raw_len / mem::size_of::<B>();

    let raw_cap = mem::size_of::<A>() * vec.capacity();
    let cap = raw_cap / mem::size_of::<B>();
    assert_eq!(raw_cap, mem::size_of::<B>() * cap);

    let ptr = vec.as_mut_ptr();
    mem::forget(vec);
    unsafe { Vec::from_raw_parts(ptr as _, len, cap) }
}

fn cast_cow<A: Pod, B: Pod>(cow: Cow<[A]>) -> Cow<[B]> {
    match cow {
        Cow::Borrowed(slice) => Cow::Borrowed(cast_slice(slice)),
        Cow::Owned(vec) => Cow::Owned(cast_vec(vec)),
    }
}
