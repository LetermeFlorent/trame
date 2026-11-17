use super::{Command, Crop};

const FALLBACK_CELL: (u16, u16) = (8, 16);

pub fn crop(command: &Command, width: u32, height: u32) -> Crop {
    let x = command.crop.x.min(width);
    let y = command.crop.y.min(height);
    let fit = |wanted: u32, room: u32| if wanted == 0 { room } else { wanted.min(room) };
    Crop {
        x,
        y,
        w: fit(command.crop.w, width - x),
        h: fit(command.crop.h, height - y),
    }
}

pub fn span(command: &Command, crop: Crop, cell: (u16, u16)) -> (u16, u16) {
    let (cell_w, cell_h) = match cell {
        (0, _) | (_, 0) => FALLBACK_CELL,
        known => known,
    };
    let (cell_w, cell_h) = (u64::from(cell_w), u64::from(cell_h));
    let (width, height) = (u64::from(crop.w.max(1)), u64::from(crop.h.max(1)));
    let (cols, rows) = (u64::from(command.cols), u64::from(command.rows));
    let (cols, rows) = match (cols, rows) {
        (0, 0) => (
            (width + u64::from(command.offset.0)).div_ceil(cell_w),
            (height + u64::from(command.offset.1)).div_ceil(cell_h),
        ),
        (0, rows) => ((rows * cell_h * width).div_ceil(height * cell_w), rows),
        (cols, 0) => (cols, (cols * cell_w * height).div_ceil(width * cell_h)),
        both => both,
    };
    (cells(cols), cells(rows))
}

fn cells(count: u64) -> u16 {
    u16::try_from(count.max(1)).unwrap_or(u16::MAX)
}
