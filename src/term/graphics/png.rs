use std::io::Cursor;

use png::{ColorType, Decoder, Transformations};

const RGB_BYTES: usize = 3;
const GRAY_ALPHA_BYTES: usize = 2;
const MAX_DECODED: usize = 128 << 20;

pub fn decode_png(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let size = reader
        .output_buffer_size()
        .filter(|&size| size <= MAX_DECODED)?;
    let mut buffer = vec![0; size];
    let info = reader.next_frame(&mut buffer).ok()?;
    let pixels = to_rgba(&buffer[..info.buffer_size()], info.color_type)?;
    Some((info.width, info.height, pixels))
}

fn to_rgba(data: &[u8], color: ColorType) -> Option<Vec<u8>> {
    let pixels = match color {
        ColorType::Rgba => data.to_vec(),
        ColorType::Rgb => data
            .as_chunks::<RGB_BYTES>()
            .0
            .iter()
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], u8::MAX])
            .collect(),
        ColorType::GrayscaleAlpha => data
            .as_chunks::<GRAY_ALPHA_BYTES>()
            .0
            .iter()
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        ColorType::Grayscale => data
            .iter()
            .flat_map(|&value| [value, value, value, u8::MAX])
            .collect(),
        ColorType::Indexed => return None,
    };
    Some(pixels)
}
