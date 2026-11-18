use std::ops::Range;

use super::scene::ImageDraw;

pub type Batch = (Range<u32>, Option<u64>);

pub fn batches(quads: usize, images: &[ImageDraw]) -> Vec<Batch> {
    let end = quads as u32;
    let mut batches: Vec<Batch> = Vec::with_capacity(images.len() * 2 + 1);
    let mut start = 0;
    for image in images.iter().filter(|image| image.quad < end) {
        if image.quad > start {
            batches.push((start..image.quad, None));
        }
        match batches.last_mut() {
            Some((range, Some(serial))) if *serial == image.serial && range.end == image.quad => {
                range.end += 1;
            }
            _ => batches.push((image.quad..image.quad + 1, Some(image.serial))),
        }
        start = image.quad + 1;
    }
    if start < end {
        batches.push((start..end, None));
    }
    batches
}
