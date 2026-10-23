use std::fmt;

pub const MAX_LETTERS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Letters {
    first: char,
    second: Option<char>,
}

impl Letters {
    pub fn parse(text: &str) -> Option<Letters> {
        let chars: Vec<char> = text.trim().chars().collect();
        if chars.iter().any(|ch| ch.is_control()) {
            return None;
        }
        match chars[..] {
            [first] => Some(Letters {
                first,
                second: None,
            }),
            [first, second] => Some(Letters {
                first,
                second: Some(second),
            }),
            _ => None,
        }
    }

    pub fn count(self) -> usize {
        1 + usize::from(self.second.is_some())
    }
}

impl fmt::Display for Letters {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.first)?;
        match self.second {
            Some(second) => write!(formatter, "{second}"),
            None => Ok(()),
        }
    }
}
