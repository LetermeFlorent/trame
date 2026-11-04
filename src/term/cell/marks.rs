use std::sync::OnceLock;

use unicode_width::UnicodeWidthChar;

const SLOTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Marks([u16; SLOTS]);

impl Marks {
    pub fn with(self, mark: char) -> Self {
        let mut marks = self;
        if let Some(code) = code_of(mark)
            && let Some(slot) = marks.0.iter_mut().find(|slot| **slot == 0)
        {
            *slot = code;
        }
        marks
    }

    pub fn is_empty(self) -> bool {
        self.0[0] == 0
    }

    pub fn chars(self) -> impl Iterator<Item = char> {
        self.0
            .into_iter()
            .take_while(|&code| code != 0)
            .map(|code| zero_width()[usize::from(code) - 1])
    }
}

fn code_of(mark: char) -> Option<u16> {
    let index = zero_width().binary_search(&mark).ok()?;
    u16::try_from(index + 1).ok()
}

fn zero_width() -> &'static [char] {
    static TABLE: OnceLock<Vec<char>> = OnceLock::new();
    TABLE.get_or_init(|| {
        (0..=u32::from(char::MAX))
            .filter_map(char::from_u32)
            .filter(|ch| ch.width() == Some(0))
            .collect()
    })
}
