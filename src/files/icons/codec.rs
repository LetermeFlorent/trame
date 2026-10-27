use std::io::{self, Cursor};

use png::{BitDepth, ColorType, Decoder, Encoder, Transformations};

use super::ICON_SIZE;

pub(super) struct Image {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) pixels: Vec<u8>,
}

pub(super) fn decode(bytes: &[u8]) -> io::Result<Image> {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(io::Error::other)?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| io::Error::other("the image is too large"))?;
    let mut buffer = vec![0; size];
    let info = reader.next_frame(&mut buffer).map_err(io::Error::other)?;
    let pixels = to_rgba(&buffer[..info.buffer_size()], info.color_type)?;
    Ok(Image {
        width: info.width,
        height: info.height,
        pixels,
    })
}

fn to_rgba(data: &[u8], color: ColorType) -> io::Result<Vec<u8>> {
    let pixels = match color {
        ColorType::Rgba => data.to_vec(),
        ColorType::Rgb => data
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], u8::MAX])
            .collect(),
        ColorType::GrayscaleAlpha => data
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        ColorType::Grayscale => data
            .iter()
            .flat_map(|&value| [value, value, value, u8::MAX])
            .collect(),
        ColorType::Indexed => return Err(io::Error::other("the palette was not expanded")),
    };
    Ok(pixels)
}

pub(super) fn encode(pixels: &[u8]) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, ICON_SIZE, ICON_SIZE);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(io::Error::other)?;
    writer.write_image_data(pixels).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
    Ok(bytes)
}
