mod dispatch;
mod editing;
mod marks;
mod scrolling;
mod selection;
mod vt;

use trame::term::Terminal;

pub fn terminal(rows: usize, cols: usize) -> Terminal {
    Terminal::new(rows, cols, 100)
}

pub fn feed(rows: usize, cols: usize, input: &str) -> Terminal {
    let mut term = terminal(rows, cols);
    term.feed(input.as_bytes());
    term
}

pub fn lines(term: &Terminal) -> Vec<String> {
    let grid = term.screen().grid();
    (0..grid.rows()).map(|row| grid.row(row).text()).collect()
}

pub fn cursor(term: &Terminal) -> (usize, usize) {
    let cursor = term.screen().cursor();
    (cursor.row, cursor.col)
}
