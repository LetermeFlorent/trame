use std::ops::Range;

use super::Row;
use crate::term::cell::Marks;

pub(super) type MarkStore = Option<Box<[(u16, Marks)]>>;

#[cfg(test)]
thread_local! {
    static MARK_LIST_REBUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl Row {
    pub fn marks(&self, col: usize) -> Marks {
        let list = self.marks.as_deref().unwrap_or_default();
        list.binary_search_by(|(at, _)| usize::from(*at).cmp(&col))
            .map_or_else(|_| Marks::default(), |index| list[index].1)
    }

    pub fn add_mark(&mut self, col: usize, mark: char) {
        let Ok(at) = u16::try_from(col) else {
            return;
        };
        let list = self.marks.as_deref_mut().unwrap_or_default();
        let index = list.partition_point(|(found, _)| *found < at);
        match list.get_mut(index) {
            Some((found, marks)) if *found == at => *marks = marks.with(mark),
            _ => {
                let marks = Marks::default().with(mark);
                if marks.is_empty() {
                    return;
                }
                let (left, right) = list.split_at(index);
                self.marks = Some([left, &[(at, marks)], right].concat().into_boxed_slice());
            }
        }
        debug_assert!(self.marks_in_order());
        self.cell_mut(col);
    }

    pub fn clear_marks(&mut self, cols: Range<usize>) {
        if self.marked_width() > cols.start {
            self.drop_marks_inside(cols);
        }
    }

    pub fn marked_width(&self) -> usize {
        self.marks
            .as_deref()
            .and_then(<[_]>::last)
            .map_or(0, |(at, _)| usize::from(*at) + 1)
    }

    pub(super) fn rotate_marks(&mut self, cols: Range<usize>, by: usize) {
        let len = cols.len();
        if len == 0 || self.marked_width() <= cols.start {
            return;
        }
        self.move_marks(|col| {
            Some(match cols.contains(&col) {
                true => cols.start + (col - cols.start + by) % len,
                false => col,
            })
        });
        if let Some(list) = self.marks.as_deref_mut() {
            list.sort_unstable_by_key(|(at, _)| *at);
        }
        debug_assert!(self.marks_in_order());
    }

    pub(super) fn marks_in_order(&self) -> bool {
        self.marks
            .as_deref()
            .unwrap_or_default()
            .is_sorted_by(|(left, _), (right, _)| left < right)
    }

    #[cold]
    fn drop_marks_inside(&mut self, cols: Range<usize>) {
        let list = self.marks.as_deref().unwrap_or_default();
        let first = list.partition_point(|(at, _)| usize::from(*at) < cols.start);
        if list
            .get(first)
            .is_some_and(|(at, _)| cols.contains(&usize::from(*at)))
        {
            self.move_marks(|col| (!cols.contains(&col)).then_some(col));
        }
    }

    fn move_marks(&mut self, to: impl Fn(usize) -> Option<usize>) {
        let Some(list) = self.marks.take() else {
            return;
        };
        #[cfg(test)]
        MARK_LIST_REBUILDS.with(|count| count.set(count.get() + 1));
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

#[cfg(test)]
mod tests;
