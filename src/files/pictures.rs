use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};

use image::ImageReader;
use image::imageops::{FilterType, resize, thumbnail};
pub use image::{Limits, RgbaImage};

use super::open;

pub const MAX_IMAGE_DIM: u32 = 8192;
pub const MAX_IMAGE_ALLOC: u64 = 256 * 1024 * 1024;
pub const MAX_IMAGE_FILE_BYTES: u64 = 64 * 1024 * 1024;
const BYTES_PER_PIXEL: u64 = 4;
const IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

pub fn image_limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIM);
    limits.max_image_height = Some(MAX_IMAGE_DIM);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC);
    limits
}

pub fn dimensions(bytes: &[u8], limits: Limits) -> io::Result<(u32, u32)> {
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    reader.limits(limits.clone());
    let (width, height) = reader.into_dimensions().map_err(io::Error::other)?;
    check_bounds(width, height, &limits)
}

pub fn decode(bytes: &[u8], limits: Limits) -> io::Result<RgbaImage> {
    let _ = dimensions(bytes, limits.clone())?;
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    reader.limits(limits);
    let image = reader.decode().map_err(io::Error::other)?;
    Ok(image.into_rgba8())
}

pub fn fit_size(src_w: u32, src_h: u32, width: u32, height: u32) -> (u32, u32) {
    if width == 0 || height == 0 || src_w == 0 || src_h == 0 {
        return (0, 0);
    }
    let scale = (width as f32 / src_w as f32).min(height as f32 / src_h as f32);
    let target_w = (src_w as f32 * scale).round().clamp(1.0, width as f32) as u32;
    let target_h = (src_h as f32 * scale).round().clamp(1.0, height as f32) as u32;
    (target_w, target_h)
}

pub fn fit_into(image: &RgbaImage, width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    let (src_w, src_h) = image.dimensions();
    let (target_w, target_h) = fit_size(src_w, src_h, width, height);
    if target_w == 0 || target_h == 0 {
        return (Vec::new(), 0, 0);
    }
    if (target_w, target_h) == (src_w, src_h) {
        return (image.as_raw().clone(), target_w, target_h);
    }
    let resized = if target_w < src_w && target_h < src_h {
        thumbnail(image, target_w, target_h)
    } else {
        resize(image, target_w, target_h, FilterType::Triangle)
    };
    (resized.into_raw(), target_w, target_h)
}

fn check_bounds(width: u32, height: u32, limits: &Limits) -> io::Result<(u32, u32)> {
    if width == 0 || height == 0 {
        return Err(io::Error::other("the image is empty"));
    }
    if limits.max_image_width.is_some_and(|max| width > max)
        || limits.max_image_height.is_some_and(|max| height > max)
    {
        return Err(io::Error::other("the image is too large"));
    }
    let bytes = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(BYTES_PER_PIXEL))
        .ok_or_else(|| io::Error::other("the image is too large"))?;
    if limits.max_alloc.is_some_and(|max| bytes > max) {
        return Err(io::Error::other("the image exceeds memory limits"));
    }
    Ok((width, height))
}

pub fn has_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| IMAGE_EXTENSIONS.iter().any(|k| ext.eq_ignore_ascii_case(k)))
}

pub fn read_image_bytes(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(io::Error::other("the image path is not a file"));
    }
    if meta.len() > MAX_IMAGE_FILE_BYTES {
        return Err(io::Error::other("the image file is too large"));
    }
    read_bounded(file, meta.len())
}

fn read_bounded(reader: impl Read, len_hint: u64) -> io::Result<Vec<u8>> {
    let cap = usize::try_from(len_hint.min(MAX_IMAGE_FILE_BYTES)).unwrap_or(0);
    let mut bytes = Vec::with_capacity(cap);
    reader
        .take(MAX_IMAGE_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_IMAGE_FILE_BYTES {
        return Err(io::Error::other("the image file is too large"));
    }
    Ok(bytes)
}

pub fn open_image_file(path: PathBuf) {
    if has_image_extension(&path) && path.is_file() {
        open::launch_path(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_bounded_rejects_stream_past_limit_even_with_zero_hint() {
        let err = read_bounded(io::repeat(0xab), 0).unwrap_err();
        assert!(err.to_string().contains("too large"));
        let ok = read_bounded(Cursor::new(&b"png"[..]), 0).expect("within limit");
        assert_eq!(ok, b"png");
    }
}
