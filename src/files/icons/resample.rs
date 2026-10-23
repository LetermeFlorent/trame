use std::ops::Range;

pub fn fit(pixels: &[u8], width: u32, height: u32, size: u32) -> Vec<u8> {
    if width == size && height == size {
        return pixels.to_vec();
    }
    let scale = size as f32 / width.max(height).max(1) as f32;
    let shown = |length: u32| ((length as f32 * scale).round() as u32).clamp(1, size);
    let (shown_width, shown_height) = (shown(width), shown(height));
    let left = (size - shown_width) / 2;
    let top = (size - shown_height) / 2;
    let mut out = vec![0; (size * size * 4) as usize];
    for y in 0..shown_height {
        let rows = span(y, shown_height, height);
        for x in 0..shown_width {
            let columns = span(x, shown_width, width);
            let pixel = average(pixels, width, columns, rows.clone());
            let at = (((top + y) * size + left + x) * 4) as usize;
            out[at..at + 4].copy_from_slice(&pixel);
        }
    }
    out
}

fn span(index: u32, shown: u32, source: u32) -> Range<u32> {
    let start = (u64::from(index) * u64::from(source) / u64::from(shown)) as u32;
    let end = (u64::from(index + 1) * u64::from(source)).div_ceil(u64::from(shown)) as u32;
    start..end.max(start + 1).min(source)
}

fn average(pixels: &[u8], width: u32, columns: Range<u32>, rows: Range<u32>) -> [u8; 4] {
    let mut sum = [0u64; 4];
    let mut count = 0u64;
    for y in rows {
        for x in columns.clone() {
            let at = ((y * width + x) * 4) as usize;
            let Some(pixel) = pixels.get(at..at + 4) else {
                continue;
            };
            let alpha = u64::from(pixel[3]);
            for (total, value) in sum.iter_mut().zip(&pixel[..3]) {
                *total += u64::from(*value) * alpha;
            }
            sum[3] += alpha;
            count += 1;
        }
    }
    let alpha = sum[3];
    if alpha == 0 {
        return [0; 4];
    }
    let channel = |index: usize| ((sum[index] + alpha / 2) / alpha) as u8;
    let coverage = ((alpha + count / 2) / count) as u8;
    [channel(0), channel(1), channel(2), coverage]
}
