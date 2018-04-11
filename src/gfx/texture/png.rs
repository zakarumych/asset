
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Read;

use hal::Backend;
use hal::format::Format;
use hal::image::Kind;

use png;

use render::{Factory, Error as RenderError};

use asset::Asset;
use gfx::texture::Texture;

#[derive(Debug)]
pub enum PngError {
    Png(png::DecodingError),
    Render(RenderError),
}

impl Display for PngError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            PngError::Png(ref err) => {
                write!(fmt, "Png error: {}", err)
            }
            PngError::Render(ref err) => {
                write!(fmt, "Render error: {}", err)
            }
        }
    }
}

impl Error for PngError {
    fn description(&self) -> &str {
        match *self {
            PngError::Png(ref err) => err.description(),
            PngError::Render(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            PngError::Png(ref err) => Some(err),
            PngError::Render(ref err) => Some(err),
        }
    }
}

pub struct PngFormat;

impl<B> Asset<PngFormat, Factory<B>> for Texture<B>
where
    B: Backend,
{
    type Error = PngError;

    fn load<R>(read: R, _format: PngFormat, factory: &mut Factory<B>) -> Result<Self, PngError>
    where
        R: Read,
    {
        let (info, mut reader) = png::Decoder::new(read).read_info().map_err(PngError::Png)?;

        let (color_type, bit_depth) = reader.output_color_type();

        let data = {
            let mut data = Vec::new();
            data.resize(reader.output_buffer_size(), 0);
            reader.next_frame(&mut data).map_err(PngError::Png)?;
            data
        };

        Texture::<B>::new(Kind::D2(info.width, info.height, 1, 1))
            .with_format(format_from_info(color_type, bit_depth))
            .with_data_width({
                let line_bytes = reader.output_line_size(info.width);
                let pixel_bytes = pixel_bytes(color_type, bit_depth);
                assert_eq!(line_bytes % pixel_bytes, 0);
                (line_bytes / pixel_bytes) as u32
            })
            .with_data(data)
            .build(factory)
            .map_err(PngError::Render)
    }
}

fn format_from_info(color_type: png::ColorType, bit_depth: png::BitDepth) -> Format {
    match color_type {
        png::ColorType::RGB => match bit_depth {
            png::BitDepth::Eight => Format::Rgb8Unorm,
            png::BitDepth::Sixteen => Format::Rgb16Unorm,
            _ => unimplemented!()
        }
        png::ColorType::RGBA => match bit_depth {
            png::BitDepth::Eight => Format::Rgba8Unorm,
            png::BitDepth::Sixteen => Format::Rgba16Unorm,
            _ => unimplemented!()
        }
        _ => unimplemented!()
    }
}

fn pixel_bytes(color_type: png::ColorType, bit_depth: png::BitDepth) -> usize {
    color_type.samples() * match bit_depth {
        png::BitDepth::One => 1,
        png::BitDepth::Two => 2,
        png::BitDepth::Four => 4,
        png::BitDepth::Eight => 8,
        png::BitDepth::Sixteen => 16,
    }
}
