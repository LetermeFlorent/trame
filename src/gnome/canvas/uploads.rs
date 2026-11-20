use gtk::cairo;
use trame::render::Upload;

const BYTES_RGBA: usize = 4;
const HALF_BYTE: u32 = 127;
const FULL_BYTE: u32 = 255;

pub struct SurfaceSlot {
    pub size: u32,
    pub surface: cairo::ImageSurface,
}

pub fn ensure_slot(slot: &mut Option<SurfaceSlot>, format: cairo::Format, size: u32) {
    let size = size.max(1);
    if slot.as_ref().is_some_and(|existing| existing.size == size) {
        return;
    }
    if let Ok(surface) = cairo::ImageSurface::create(format, size as i32, size as i32) {
        *slot = Some(SurfaceSlot { size, surface });
    }
}

pub fn write_mask(slot: Option<&mut SurfaceSlot>, upload: &Upload) {
    let Some(slot) = slot else { return };
    let r = upload.region;
    let width = r.width as usize;
    let stride = slot.surface.stride().max(0) as usize;
    let _ = slot.surface.with_data(|dest| {
        for row in 0..r.height as usize {
            let src_at = row * width;
            let dst_at = (r.y as usize + row) * stride + r.x as usize;
            copy_slice(&upload.data, src_at, dest, dst_at, width);
        }
    });
}

fn copy_slice(src: &[u8], src_at: usize, dst: &mut [u8], dst_at: usize, len: usize) {
    if let (Some(s), Some(d)) = (src.get(src_at..src_at + len), dst.get_mut(dst_at..dst_at + len))
    {
        d.copy_from_slice(s);
    }
}

pub fn write_color(slot: Option<&mut SurfaceSlot>, upload: &Upload) {
    let Some(slot) = slot else { return };
    let r = upload.region;
    let row_bytes = r.width as usize * BYTES_RGBA;
    let stride = slot.surface.stride().max(0) as usize;
    let _ = slot.surface.with_data(|dest| {
        for row in 0..r.height as usize {
            let src_at = row * row_bytes;
            let dst_at = (r.y as usize + row) * stride + r.x as usize * BYTES_RGBA;
            write_color_row(&upload.data, src_at, dest, dst_at, row_bytes);
        }
    });
}

fn write_color_row(src: &[u8], src_at: usize, dst: &mut [u8], dst_at: usize, len: usize) {
    let (Some(s), Some(d)) = (src.get(src_at..src_at + len), dst.get_mut(dst_at..dst_at + len))
    else {
        return;
    };
    for (rgba, out) in s.chunks_exact(BYTES_RGBA).zip(d.chunks_exact_mut(BYTES_RGBA)) {
        out.copy_from_slice(&premultiply(rgba));
    }
}

fn premultiply(rgba: &[u8]) -> [u8; 4] {
    let a = u32::from(rgba[3]);
    let scale = |c: u8| ((u32::from(c) * a + HALF_BYTE) / FULL_BYTE) as u8;
    let word = (a << 24)
        | (u32::from(scale(rgba[0])) << 16)
        | (u32::from(scale(rgba[1])) << 8)
        | u32::from(scale(rgba[2]));
    word.to_ne_bytes()
}
