use std::cell::Cell;

use super::{MARK_LIST_REBUILDS, Row};
use crate::term::cell::Marks;

const ACUTE: char = '\u{301}';
const DIAERESIS: char = '\u{308}';
const FULL_ROW_PROBE_STEP: usize = 32;

fn list_address(row: &Row) -> Option<*const (u16, Marks)> {
    row.marks.as_deref().map(<[_]>::as_ptr)
}

fn mark_list_rebuilds() -> usize {
    MARK_LIST_REBUILDS.with(Cell::get)
}

fn out_of_order() -> Row {
    let mut row = Row::default();
    let acute = Marks::default().with(ACUTE);
    row.marks = Some([(5, acute), (2, acute)].into());
    row
}

#[test]
fn unicode_marks_stay_put_while_cells_beside_them_are_written() {
    let mut row = Row::default();
    row.add_mark(1, ACUTE);
    row.add_mark(4, DIAERESIS);
    let list = list_address(&row);
    row.clear_marks(2..4);
    row.clear_marks(5..usize::MAX);
    row.rotate_marks(6..9, 1);
    assert_eq!(list_address(&row), list);
    assert_eq!(row.marks(1), Marks::default().with(ACUTE));
    assert_eq!(row.marks(4), Marks::default().with(DIAERESIS));
}

#[test]
fn unicode_marks_join_the_list_of_their_cell() {
    let mut row = Row::default();
    row.add_mark(2, ACUTE);
    row.add_mark(0, DIAERESIS);
    row.add_mark(2, DIAERESIS);
    assert_eq!(row.marks.as_deref().map(<[_]>::len), Some(2));
    assert_eq!(row.marks(2), Marks::default().with(ACUTE).with(DIAERESIS));
    assert_eq!(row.marks(0), Marks::default().with(DIAERESIS));
    assert_eq!(row.len(), 3);
    row.clear_marks(0..1);
    assert_eq!(row.marks(0), Marks::default());
    assert_eq!(row.marks.as_deref().map(<[_]>::len), Some(1));
    row.clear_marks(2..3);
    assert_eq!(list_address(&row), None);
}

#[test]
fn unicode_marks_moved_out_of_order_keep_the_marked_width() {
    let mut row = Row::default();
    row.add_mark(5, ACUTE);
    row.add_mark(2, DIAERESIS);
    assert_eq!(row.marked_width(), 6);
    row.rotate_marks(2..6, 1);
    assert_eq!(row.marks(3), Marks::default().with(DIAERESIS));
    assert_eq!(row.marks(2), Marks::default().with(ACUTE));
    assert_eq!(row.marked_width(), 4);
    row.clear_marks(3..usize::MAX);
    assert_eq!(row.marked_width(), 3);
}

#[test]
fn unicode_marks_of_a_full_row_stay_put_while_cells_between_them_are_written() {
    let mut row = Row::default();
    let acute = Marks::default().with(ACUTE);
    row.marks = Some((1..=u16::MAX).step_by(2).map(|at| (at, acute)).collect());
    let list = list_address(&row);
    let rebuilds = mark_list_rebuilds();
    for col in (0..usize::from(u16::MAX)).step_by(FULL_ROW_PROBE_STEP) {
        row.clear_marks(col..col + 1);
    }
    assert_eq!(list_address(&row), list);
    assert_eq!(mark_list_rebuilds(), rebuilds);
    assert_eq!(row.marks(usize::from(u16::MAX)), acute);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "marks_in_order")]
fn unicode_marks_added_to_an_unsorted_list_trip_the_order_guard() {
    out_of_order().add_mark(9, DIAERESIS);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "marks_in_order")]
fn unicode_marks_refilled_from_an_unsorted_row_trip_the_order_guard() {
    Row::default().refill_from(&mut out_of_order());
}
