use std::{iter, mem};

pub const MAX_CHARS: usize = 120;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextField {
    text: String,
    caret: usize,
    selected: bool,
}

impl TextField {
    pub fn new(text: &str) -> Self {
        let mut field = Self::default();
        field.insert(text);
        field
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn select_all(&mut self) {
        self.selected = !self.text.is_empty();
        self.caret = self.text.len();
    }

    pub fn before_caret(&self) -> &str {
        &self.text[..self.caret]
    }

    pub fn masked(&self, mask: char) -> Self {
        let count = |text: &str| text.chars().count();
        Self {
            text: iter::repeat_n(mask, count(&self.text)).collect(),
            caret: mask.len_utf8() * count(self.before_caret()),
            selected: self.selected,
        }
    }

    pub fn insert(&mut self, typed: &str) {
        let kept = if self.selected {
            0
        } else {
            self.text.chars().count()
        };
        let accepted: String = typed
            .chars()
            .filter(|ch| !ch.is_control())
            .take(MAX_CHARS.saturating_sub(kept))
            .collect();
        if accepted.is_empty() {
            return;
        }
        self.clear_selection();
        self.text.insert_str(self.caret, &accepted);
        self.caret += accepted.len();
    }

    pub fn backspace(&mut self) {
        if self.clear_selection() {
            return;
        }
        let Some(start) = self.previous_boundary() else {
            return;
        };
        self.text.replace_range(start..self.caret, "");
        self.caret = start;
    }

    pub fn delete(&mut self) {
        if self.clear_selection() {
            return;
        }
        let Some(end) = self.next_boundary() else {
            return;
        };
        self.text.replace_range(self.caret..end, "");
    }

    pub fn left(&mut self) {
        if mem::take(&mut self.selected) {
            return self.home();
        }
        if let Some(start) = self.previous_boundary() {
            self.caret = start;
        }
    }

    pub fn right(&mut self) {
        if mem::take(&mut self.selected) {
            return self.end();
        }
        if let Some(end) = self.next_boundary() {
            self.caret = end;
        }
    }

    pub fn home(&mut self) {
        self.selected = false;
        self.caret = 0;
    }

    pub fn end(&mut self) {
        self.selected = false;
        self.caret = self.text.len();
    }

    fn clear_selection(&mut self) -> bool {
        if !mem::take(&mut self.selected) {
            return false;
        }
        self.text.clear();
        self.caret = 0;
        true
    }

    fn previous_boundary(&self) -> Option<usize> {
        self.before_caret()
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
    }

    fn next_boundary(&self) -> Option<usize> {
        let ch = self.text[self.caret..].chars().next()?;
        Some(self.caret + ch.len_utf8())
    }
}
