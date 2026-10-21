use super::Grid;
use super::row::Row;
use crate::term::cell::Flags;

impl Grid {
    pub fn selected_text(&self) -> Option<String> {
        let selection = self.selection.as_ref().filter(|found| !found.is_empty())?;
        let (start, end) = (selection.start(), selection.end());
        let first = start.line.max(self.first_line());
        let last = end.line.min(self.end_line() - 1);
        let mut text = String::new();
        for line in first..=last {
            let Some(row) = self.line(line) else {
                continue;
            };
            let from = if line == start.line { start.col } else { 0 };
            let to = if line == end.line {
                end.col
            } else {
                self.cols - 1
            };
            let joined = row.wrapped && line < end.line;
            let piece = slice(row, from, to);
            text.push_str(if joined {
                &piece
            } else {
                piece.trim_end_matches(' ')
            });
            if line < last && !joined {
                text.push('\n');
            }
        }
        Some(text)
    }
}

fn slice(row: &Row, from: usize, to: usize) -> String {
    (from..=to)
        .map(|col| row.cell(col))
        .filter(|cell| !cell.flags.contains(Flags::WIDE_SPACER))
        .map(|cell| if cell.ch == '\0' { ' ' } else { cell.ch })
        .collect()
}
