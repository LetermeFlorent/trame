use std::io;

use png::Compression;

use super::ICON_SIZE;
use crate::files::capture::encode_rgba;
use crate::term::graphics::decode_png;

pub(super) struct Image {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) pixels: Vec<u8>,
}

pub(super) fn decode(bytes: &[u8]) -> io::Result<Image> {
    let (width, height, pixels) =
        decode_png(bytes).ok_or_else(|| io::Error::other("the image could not be decoded"))?;
    Ok(Image {
        width,
        height,
        pixels,
    })
}

pub(super) fn encode(pixels: &[u8]) -> io::Result<Vec<u8>> {
    encode_rgba(ICON_SIZE, ICON_SIZE, pixels, Compression::default())
}
