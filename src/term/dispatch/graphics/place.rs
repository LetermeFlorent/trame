use crate::term::graphics::{Action, Anchor, Command, Error, crop, span};
use crate::term::screen::Screen;

impl Screen {
    pub(in crate::term::dispatch) fn place(&mut self, command: &Command, id: u32) {
        let Some(image) = self.graphics.get(id) else {
            return self.graphics_reply(command, id, Err(Error::NoEnt));
        };
        let crop = crop(command, image.width, image.height);
        let (cols, rows) = span(command, crop, self.cell_pixels);
        if command.virtual_ {
            self.graphics.virtuals.insert(id, (cols, rows));
        } else {
            let anchor = Anchor {
                col: u16::try_from(self.cursor.col).unwrap_or(u16::MAX),
                image: id,
                placement: command.placement,
                cols,
                rows,
                crop,
                offset: command.offset,
                z: command.z,
            };
            self.anchor_at_cursor(anchor, command.cursor_stays);
        }
        if command.action == Action::Place {
            self.graphics_reply(command, id, Ok(()));
        }
    }

    pub(in crate::term::dispatch) fn anchor_at_cursor(
        &mut self,
        anchor: Anchor,
        cursor_stays: bool,
    ) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        let (cols, rows) = (usize::from(anchor.cols), anchor.rows);
        self.graphics.note_rows(rows);
        let grid = self.grid_mut();
        grid.remove_anchors(false, |_, found| found.same_placement(&anchor));
        grid.row_mut(row).add_anchor(anchor);
        if cursor_stays {
            return;
        }
        for _ in 1..rows {
            self.index();
        }
        self.goto_col(col + cols);
    }
}
