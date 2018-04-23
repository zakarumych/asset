
use std::io::Read;

use failure::Error;
use hal::Backend;
use hal::format::Format;
use hal::image::Kind;

use png;

use render::Factory;

use asset::AssetLoader;
use texture::Texture;

pub struct PngFormat;

impl<B> AssetLoader<Texture<B>, PngFormat> for Factory<B>
where
    B: Backend,
{
    type Error = Error;

    fn load<R>(&mut self, format: PngFormat, reader: R) -> Result<Texture<B>, Error>
    where
        R: Read,
    {
        let PngFormat = format;

        let (info, mut reader) = png::Decoder::new(reader).read_info()?;

        let (color_type, bit_depth) = reader.output_color_type();

        let data = {
            let mut data = Vec::new();
            data.resize(reader.output_buffer_size(), 0);
            reader.next_frame(&mut data)?;
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
            .build(self)
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
    } / 8
}
