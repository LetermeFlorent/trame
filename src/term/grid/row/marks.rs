use std::ops::Range;

use super::Row;
use crate::term::cell::Marks;

pub(super) type MarkStore = Option<Box<[(u16, Marks)]>>;

impl Row {
    pub fn marks(&self, col: usize) -> Marks {
        self.marks
            .iter()
            .flatten()
            .find(|(at, _)| usize::from(*at) == col)
            .map_or_else(Marks::default, |(_, marks)| *marks)
    }

    pub fn add_mark(&mut self, col: usize, mark: char) {
        let next = self.marks(col).with(mark);
        let Ok(at) = u16::try_from(col) else {
            return;
        };
        if next.is_empty() {
            return;
        }
        self.cell_mut(col);
        let mut list = self.marks.take().map(Vec::from).unwrap_or_default();
        list.retain(|(found, _)| *found != at);
        list.push((at, next));
        self.marks = Some(list.into_boxed_slice());
    }

    pub fn clear_marks(&mut self, cols: Range<usize>) {
        self.move_marks(|col| (!cols.contains(&col)).then_some(col));
    }

    pub fn marked_width(&self) -> usize {
        self.marks
            .iter()
            .flatten()
            .map(|(at, _)| usize::from(*at) + 1)
            .max()
            .unwrap_or(0)
    }

    pub(super) fn rotate_marks(&mut self, cols: Range<usize>, by: usize) {
        let len = cols.len();
        if len == 0 {
            return;
        }
        self.move_marks(|col| {
            Some(match cols.contains(&col) {
                true => cols.start + (col - cols.start + by) % len,
                false => col,
            })
        });
    }

    fn move_marks(&mut self, to: impl Fn(usize) -> Option<usize>) {
        let Some(list) = self.marks.take() else {
            return;
        };
        let list: Vec<_> = list
            .into_iter()
            .filter_map(|(at, marks)| {
                let moved = to(usize::from(at)).and_then(|col| u16::try_from(col).ok())?;
                Some((moved, marks))
            })
            .collect();
        self.marks = (!list.is_empty()).then(|| list.into_boxed_slice());
    }
}
