use super::Grid;
use super::row::printable;

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
            let piece = printable((from..=to).map(|col| row.cell(col)));
            text.push_str(if joined {
                &piece
            } else {
                piece.trim_end_matches(' ')
            });
            if line < last && !joined {
                text.push('\n');
            }
        }
        (!text.is_empty()).then_some(text)
    }
}
