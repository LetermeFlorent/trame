use super::tile_char;
use crate::term::cell::{Cell, Color, Flags};
use crate::term::grid::Row;
use crate::term::screen::Screen;

impl Screen {
    pub(super) fn place_image(&mut self, id: u32, rows: usize, cols: usize) {
        let grid_cols = self.cols();
        if self.cursor.col > 0 && self.cursor.col + cols > grid_cols {
            self.carriage_return();
            self.index();
        }
        let start_col = self.cursor.col.min(grid_cols.saturating_sub(cols));
        let fg = Color::Rgb(
            ((id >> 16) & 0xFF) as u8,
            ((id >> 8) & 0xFF) as u8,
            (id & 0xFF) as u8,
        );
        let bg = self.cursor.template.bg;
        for row_idx in 0..rows {
            let row = self.cursor.row;
            let template = self.cursor.template;
            let line = self.grid_mut().row_mut(row);
            clear_wide_edges(line, start_col, cols, &template);
            line.clear_marks(start_col..start_col + cols);
            for col_idx in 0..cols {
                *line.cell_mut(start_col + col_idx) = Cell {
                    ch: tile_char(row_idx, col_idx),
                    fg,
                    bg,
                    flags: Flags::IMAGE,
                    link: 0,
                };
            }
            line.wrapped = false;
            self.carriage_return();
            self.index();
        }
    }
}

fn clear_wide_edges(line: &mut Row, start_col: usize, cols: usize, template: &Cell) {
    if start_col > 0 && line.cell(start_col).flags.contains(Flags::WIDE_SPACER) {
        let leader = line.cell_mut(start_col - 1);
        *leader = Cell::blank_with(template);
        line.clear_marks(start_col - 1..start_col);
    }
    let last = start_col + cols.saturating_sub(1);
    if cols > 0 && line.cell(last).flags.contains(Flags::WIDE) {
        let spacer = line.cell_mut(last + 1);
        *spacer = Cell::blank_with(template);
    }
}
