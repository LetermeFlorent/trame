use super::Row;
use crate::term::cell::Cell;

const SPARE_CELLS_DIVISOR: usize = 8;

impl Row {
    pub(in crate::term::grid) fn reset(&mut self, cols: usize, template: &Cell) {
        self.empty_with_room_for(cols);
        self.marks = None;
        self.wrapped = false;
        let blank = Cell::blank_with(template);
        if blank != Cell::default() {
            self.cells.resize(cols, blank);
        }
    }

    pub(in crate::term::grid) fn refill_from(&mut self, source: &mut Row) {
        source.trim();
        self.empty_with_room_for(source.cells.len());
        self.cells.extend_from_slice(&source.cells);
        self.marks = source.marks.take();
        self.wrapped = source.wrapped;
    }

    fn empty_with_room_for(&mut self, cells: usize) {
        if (cells..=cells + cells / SPARE_CELLS_DIVISOR).contains(&self.cells.capacity()) {
            self.cells.clear();
        } else {
            self.cells = Vec::with_capacity(cells);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Row, SPARE_CELLS_DIVISOR};
    use crate::term::cell::{Cell, Color, Marks};
    use crate::term::grid::Grid;

    fn row_of(width: usize) -> Row {
        let mut row = Row::default();
        for col in 0..width {
            row.cell_mut(col).ch = 'x';
        }
        row
    }

    fn assert_fits(row: &Row, used: usize) {
        let room = row.cells.capacity();
        assert!(
            (used..=used + used / SPARE_CELLS_DIVISOR).contains(&room),
            "{room} cells of room for {used} used"
        );
    }

    #[test]
    fn a_refilled_row_keeps_its_buffer_only_while_it_stays_close_to_the_text() {
        let mut kept = Row::default();
        kept.refill_from(&mut row_of(80));
        assert_fits(&kept, 80);
        let room = kept.cells.capacity();
        kept.refill_from(&mut row_of(75));
        assert_eq!(kept.cells.capacity(), room);
        kept.refill_from(&mut row_of(20));
        assert_fits(&kept, 20);
        assert_eq!(kept.text(), "x".repeat(20));
    }

    fn painted() -> Cell {
        Cell {
            bg: Color::Indexed(1),
            ..Cell::default()
        }
    }

    #[test]
    fn a_reset_row_is_blank_in_the_template_background_and_fits_its_buffer_to_the_width() {
        let mut row = row_of(100);
        row.wrapped = true;
        row.add_mark(3, '\u{301}');
        assert_ne!(row.marks(3), Marks::default());
        row.reset(40, &painted());
        assert_eq!(row.cells(), vec![Cell::blank_with(&painted()); 40]);
        assert_fits(&row, 40);
        assert!(!row.wrapped);
        assert_eq!(row.marks(3), Marks::default());
        let room = row.cells.capacity();
        row.reset(38, &Cell::default());
        assert!(row.cells().is_empty());
        assert_eq!(row.cells.capacity(), room);
    }

    #[test]
    fn a_painted_reset_of_a_narrower_buffer_allocates_just_the_width() {
        let (narrow, wide) = (32, 40);
        let mut row = Row::default();
        row.cells.reserve_exact(narrow);
        row.reset(wide, &painted());
        assert_eq!(row.cells(), vec![Cell::blank_with(&painted()); wide]);
        assert_fits(&row, wide);
    }

    #[test]
    fn screen_rows_give_their_room_back_once_scrolled_after_a_narrowing() {
        let (rows, wide, narrow) = (3, 100, 40);
        let mut grid = Grid::new(rows, wide, 10);
        for row in grid.lines.iter_mut() {
            *row = row_of(wide);
        }
        grid.resize(rows, narrow, &mut 0);
        grid.scroll_up(0, rows - 1, rows, &Cell::default());
        for row in grid.lines.iter().skip(grid.history_len()) {
            assert_fits(row, narrow);
        }
    }
}
