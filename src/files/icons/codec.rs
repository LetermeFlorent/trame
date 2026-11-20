use std::io;

use png::Compression;

use super::ICON_SIZE;
use crate::files::capture::encode_rgba;
use crate::files::pictures::{decode as decode_picture, image_limits};

pub(super) struct Image {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) pixels: Vec<u8>,
}

pub(super) fn decode(bytes: &[u8]) -> io::Result<Image> {
    let image = decode_picture(bytes, image_limits())?;
    let (width, height) = image.dimensions();
    Ok(Image {
        width,
        height,
        pixels: image.into_raw(),
    })
}

pub(super) fn encode(pixels: &[u8]) -> io::Result<Vec<u8>> {
    encode_rgba(ICON_SIZE, ICON_SIZE, pixels, Compression::default())
}
