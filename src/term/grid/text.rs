use super::Grid;
use super::row::printable;

impl Grid {
    pub fn selected_text(&self) -> Option<String> {
        let mut text = String::new();
        self.any_selected_piece(|piece| {
            text.push_str(piece);
            false
        });
        (!text.is_empty()).then_some(text)
    }

    pub fn has_selected_text(&self) -> bool {
        self.any_selected_piece(|piece| !piece.is_empty())
    }

    fn any_selected_piece(&self, mut matches: impl FnMut(&str) -> bool) -> bool {
        let Some(selection) = self.selection.as_ref().filter(|found| !found.is_empty()) else {
            return false;
        };
        let (start, end) = (selection.start(), selection.end());
        let first = start.line.max(self.first_line());
        let last = end.line.min(self.end_line() - 1);
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
            let kept = if joined {
                &piece
            } else {
                piece.trim_end_matches(' ')
            };
            if matches(kept) || line < last && !joined && matches("\n") {
                return true;
            }
        }
        false
    }
}
