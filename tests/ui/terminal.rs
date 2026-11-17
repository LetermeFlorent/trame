use trame::geometry::{Point, Rect};
use trame::term::{CursorShape, SelectionKind, Terminal};
use trame::text::CellMetrics;
use trame::ui::terminal::{CursorPlace, ScreenCopy};

use crate::{frame, text_system, workspace};

const ROWS: usize = 4;
const COLS: usize = 10;

fn typed(input: &str) -> Terminal {
    let mut term = Terminal::new(ROWS, COLS, 100);
    term.feed(input.as_bytes());
    term
}

fn cell_metrics() -> CellMetrics {
    text_system().cell_metrics()
}

#[test]
fn the_cursor_sits_on_the_next_free_cell() {
    let place = CursorPlace::locate(typed("ab").screen()).expect("a visible cursor");
    assert_eq!((place.row, place.col, place.columns), (0, 2, 1));
}

#[test]
fn a_hidden_cursor_has_no_place() {
    assert!(CursorPlace::locate(typed("\x1b[?25l").screen()).is_none());
}

#[test]
fn the_cursor_on_a_wide_spacer_moves_back_to_the_wide_cell() {
    let place = CursorPlace::locate(typed("a\u{65E5}\x1b[1;3H").screen()).expect("a cursor");
    assert_eq!((place.col, place.columns), (1, 2));
}

#[test]
fn covers_spans_every_column_of_the_cursor() {
    let place = CursorPlace::locate(typed("a\u{65E5}\x1b[1;3H").screen()).expect("a cursor");
    assert!(place.covers(0, 1));
    assert!(place.covers(0, 2));
    assert!(!place.covers(0, 0));
    assert!(!place.covers(0, 3));
    assert!(!place.covers(1, 1));
}

#[test]
fn a_copy_selects_what_the_scrolled_screen_selects() {
    let mut term = typed("1\r\n2\r\n3\r\n4\r\n5 five\r\n6 six");
    let screen = term.screen_mut();
    screen.scroll_display(1);
    screen.start_selection(SelectionKind::Simple, 1, 3);
    screen.extend_selection(3, 1);
    let copy = ScreenCopy::of(screen, 0.5);
    let grid = screen.grid();
    for row in -1..=ROWS as isize {
        for col in 0..COLS {
            let shown = grid.is_selected_in_view(row, col);
            assert_eq!(copy.is_selected(row, col), shown, "row {row} col {col}");
        }
    }
}

#[test]
fn rect_scales_the_cell_by_row_column_and_width() {
    let metrics = cell_metrics();
    let place = CursorPlace {
        row: 2,
        col: 3,
        columns: 2,
        shape: CursorShape::Block,
    };
    let rect = place.rect(Point::new(5.0, 7.0), &metrics);
    assert_eq!(
        rect,
        Rect::new(
            5.0 + 3.0 * metrics.width,
            7.0 + 2.0 * metrics.height,
            2.0 * metrics.width,
            metrics.height
        )
    );
}

#[test]
fn grid_size_counts_whole_cells_and_keeps_at_least_one() {
    let mut text = text_system();
    let metrics = text.cell_metrics();
    let layout = frame(&mut text, &workspace());
    let pane = &layout.panes[0];
    let (rows, cols) = pane.grid_size(&metrics);
    assert_eq!(
        rows,
        (pane.grid_area.height / metrics.height).floor() as usize
    );
    assert_eq!(
        cols,
        (pane.grid_area.width / metrics.width).floor() as usize
    );
    let mut tiny = pane.clone();
    tiny.grid_area = Rect::new(0.0, 0.0, 1.0, 1.0);
    assert_eq!(tiny.grid_size(&metrics), (1, 1));
}
