pub const MAX_CHARS: usize = 120;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextField {
    text: String,
    caret: usize,
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

    pub fn before_caret(&self) -> &str {
        &self.text[..self.caret]
    }

    pub fn insert(&mut self, typed: &str) {
        let room = MAX_CHARS.saturating_sub(self.text.chars().count());
        let accepted: String = typed
            .chars()
            .filter(|ch| !ch.is_control())
            .take(room)
            .collect();
        self.text.insert_str(self.caret, &accepted);
        self.caret += accepted.len();
    }

    pub fn backspace(&mut self) {
        let Some(start) = self.previous_boundary() else {
            return;
        };
        self.text.replace_range(start..self.caret, "");
        self.caret = start;
    }

    pub fn delete(&mut self) {
        let Some(end) = self.next_boundary() else {
            return;
        };
        self.text.replace_range(self.caret..end, "");
    }

    pub fn left(&mut self) {
        if let Some(start) = self.previous_boundary() {
            self.caret = start;
        }
    }

    pub fn right(&mut self) {
        if let Some(end) = self.next_boundary() {
            self.caret = end;
        }
    }

    pub fn home(&mut self) {
        self.caret = 0;
    }

    pub fn end(&mut self) {
        self.caret = self.text.len();
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
