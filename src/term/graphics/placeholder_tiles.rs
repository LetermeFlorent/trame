use super::placeholder::{Run, resolve_row};
use super::span::cell_size;
use super::{Image, ImageTile, Layer};
use crate::term::screen::Screen;

struct Fit {
    origin: (f32, f32),
    size: (f32, f32),
    cell: (f32, f32),
}

impl Fit {
    fn new(image: &Image, grid: (u16, u16), cell: (u16, u16)) -> Self {
        let cell = (f32::from(cell.0), f32::from(cell.1));
        let room = (f32::from(grid.0) * cell.0, f32::from(grid.1) * cell.1);
        let (width, height) = (image.width.max(1) as f32, image.height.max(1) as f32);
        let scale = (room.0 / width).min(room.1 / height);
        let size = (width * scale, height * scale);
        Self {
            origin: ((room.0 - size.0) / 2.0, (room.1 - size.1) / 2.0),
            size,
            cell,
        }
    }

    fn tile(&self, image: &Image, run: &Run, row: isize) -> Option<ImageTile> {
        let left = f32::from(run.img_col) * self.cell.0;
        let right = left + run.len as f32 * self.cell.0;
        let top = f32::from(run.img_row) * self.cell.1;
        let (x0, x1) = clip(left, right, self.origin.0, self.size.0)?;
        let (y0, y1) = clip(top, top + self.cell.1, self.origin.1, self.size.1)?;
        let u = |x: f32| ((x - self.origin.0) / self.size.0).clamp(0.0, 1.0);
        let v = |y: f32| ((y - self.origin.1) / self.size.1).clamp(0.0, 1.0);
        Some(ImageTile {
            image: image.clone(),
            uv: [u(x0), v(y0), u(x1), v(y1)],
            row,
            col: run.start,
            cols: (x1 - x0) / self.cell.0,
            rows: (y1 - y0) / self.cell.1,
            offset: (pixels(x0 - left), pixels(y0 - top)),
            layer: Layer::UnderText,
        })
    }
}

fn clip(start: f32, end: f32, origin: f32, size: f32) -> Option<(f32, f32)> {
    let (from, to) = (start.max(origin), end.min(origin + size));
    (to > from).then_some((from, to))
}

fn pixels(length: f32) -> u16 {
    length.round() as u16
}

impl Screen {
    pub(super) fn placeholder_tiles(
        &self,
        first_line: u64,
        rows: usize,
        tiles: &mut Vec<ImageTile>,
    ) {
        if self.graphics.virtuals.is_empty() {
            return;
        }
        let grid = self.grid();
        let cell = cell_size(self.cell_pixels);
        for line in first_line..first_line + rows as u64 {
            let Some(row) = grid.line(line) else {
                continue;
            };
            let at = (line - first_line) as isize;
            for run in resolve_row(row, grid.cols()) {
                let Some(span) = self.graphics.virtual_span(run.id) else {
                    continue;
                };
                if let Some(image) = self.graphics.get(run.id) {
                    tiles.extend(Fit::new(image, span, cell).tile(image, &run, at));
                }
            }
        }
    }
}
