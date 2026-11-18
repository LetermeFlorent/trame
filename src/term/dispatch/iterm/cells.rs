use super::args::{FileArgs, Size};

const FALLBACK_CELL: (u32, u32) = (8, 16);
const WHOLE: u64 = 100;

#[derive(Debug, Clone, Copy)]
pub(super) struct Geometry {
    pub(super) image: (u32, u32),
    pub(super) cell: (u16, u16),
    pub(super) screen: (usize, usize),
}

pub(super) fn cells(args: &FileArgs, geometry: Geometry) -> (u16, u16) {
    let (cell_w, cell_h) = match geometry.cell {
        (0, _) | (_, 0) => FALLBACK_CELL,
        (w, h) => (u32::from(w), u32::from(h)),
    };
    let (image_w, image_h) = (geometry.image.0.max(1), geometry.image.1.max(1));
    let (screen_cols, screen_rows) = geometry.screen;
    let width = axis(args.width, image_w, cell_w, screen_cols);
    let height = axis(args.height, image_h, cell_h, screen_rows);
    let pixels_w = u64::from(image_w) * u64::from(cell_h);
    let pixels_h = u64::from(image_h) * u64::from(cell_w);
    let (cols, rows) = match (width, height) {
        (Ok(cols), Err(_)) if args.preserve => (cols, (cols * pixels_h).div_ceil(pixels_w)),
        (Err(_), Ok(rows)) if args.preserve => ((rows * pixels_w).div_ceil(pixels_h), rows),
        (cols, rows) => (cols.unwrap_or_else(|n| n), rows.unwrap_or_else(|n| n)),
    };
    (clamp(cols, screen_cols), clamp(rows, screen_rows))
}

fn axis(size: Size, natural: u32, cell: u32, room: usize) -> Result<u64, u64> {
    match size {
        Size::Auto => Err(u64::from(natural.div_ceil(cell))),
        Size::Cells(count) => Ok(u64::from(count)),
        Size::Pixels(pixels) => Ok(u64::from(pixels.div_ceil(cell))),
        Size::Percent(share) => Ok((room as u64 * u64::from(share)).div_ceil(WHOLE)),
    }
}

fn clamp(count: u64, room: usize) -> u16 {
    let room = u64::try_from(room).unwrap_or(u64::MAX);
    u16::try_from(count.clamp(1, room.max(1))).unwrap_or(u16::MAX)
}
