use std::borrow::Cow;
use std::ops::Range;

use crate::term::graphics::Image;

pub const MAX_TEXTURE_SIDE: u32 = 8192;

pub fn downscaled_size(width: u32, height: u32, limit: u32) -> (u32, u32) {
    let (width, height) = (width.max(1), height.max(1));
    let limit = limit.max(1);
    if width <= limit && height <= limit {
        return (width, height);
    }
    let scale = (f64::from(limit) / f64::from(width)).min(f64::from(limit) / f64::from(height));
    let fit = |side: u32| ((f64::from(side) * scale).round() as u32).clamp(1, limit);
    (fit(width), fit(height))
}

pub fn downscale(image: &Image, limit: u32) -> ((u32, u32), Cow<'_, [u8]>) {
    let source = (image.width.max(1), image.height.max(1));
    let size = downscaled_size(source.0, source.1, limit);
    let expected = source.0 as usize * source.1 as usize * 4;
    if image.pixels.len() < expected {
        return ((1, 1), Cow::Owned(vec![0; 4]));
    }
    if size == source {
        return (size, Cow::Borrowed(&image.pixels[..expected]));
    }
    (size, Cow::Owned(box_filter(&image.pixels, source, size)))
}

fn box_filter(pixels: &[u8], source: (u32, u32), size: (u32, u32)) -> Vec<u8> {
    let mut out = Vec::with_capacity(size.0 as usize * size.1 as usize * 4);
    for y in 0..size.1 {
        let rows = span(y, size.1, source.1);
        for x in 0..size.0 {
            let cols = span(x, size.0, source.0);
            out.extend_from_slice(&average(pixels, source.0, &rows, &cols));
        }
    }
    out
}

fn span(index: u32, size: u32, source: u32) -> Range<u32> {
    let start = (u64::from(index) * u64::from(source) / u64::from(size)) as u32;
    let end = (u64::from(index + 1) * u64::from(source) / u64::from(size)) as u32;
    start..end.max(start + 1)
}

fn average(pixels: &[u8], stride: u32, rows: &Range<u32>, cols: &Range<u32>) -> [u8; 4] {
    let mut sum = [0u64; 4];
    for row in rows.clone() {
        let start = (row as usize * stride as usize + cols.start as usize) * 4;
        let end = (row as usize * stride as usize + cols.end as usize) * 4;
        for pixel in pixels[start..end].as_chunks::<4>().0 {
            for (total, channel) in sum.iter_mut().zip(pixel) {
                *total += u64::from(*channel);
            }
        }
    }
    let count = (rows.len() * cols.len()).max(1) as u64;
    sum.map(|total| (total / count) as u8)
}
