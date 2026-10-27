use winit::keyboard::{KeyCode, NamedKey};

use super::names::{NAMED, NUMPAD};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Chord {
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
    pub logo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trigger {
    Char(char),
    Digit(u8),
    Numpad(KeyCode),
    Named(NamedKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Combo {
    pub chord: Chord,
    pub trigger: Trigger,
}

const MODIFIERS: [&str; 4] = ["Ctrl", "Shift", "Alt", "Super"];

impl Chord {
    pub(super) const fn new(control: bool, shift: bool, alt: bool, logo: bool) -> Self {
        Self {
            control,
            shift,
            alt,
            logo,
        }
    }

    pub fn flags(self) -> [bool; 4] {
        [self.control, self.shift, self.alt, self.logo]
    }

    fn set(&mut self, name: &str) -> bool {
        let flag = match name.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => &mut self.control,
            "shift" => &mut self.shift,
            "alt" | "option" => &mut self.alt,
            "super" | "cmd" | "win" | "logo" | "meta" => &mut self.logo,
            _ => return false,
        };
        let fresh = !*flag;
        *flag = true;
        fresh
    }
}

impl Combo {
    pub const fn new(chord: Chord, trigger: Trigger) -> Self {
        Self { chord, trigger }
    }

    pub fn parse(text: &str) -> Option<Self> {
        let (mods, key) = split(text.trim());
        let mut chord = Chord::default();
        let all_known = mods
            .split('+')
            .filter(|name| !name.is_empty())
            .all(|name| chord.set(name.trim()));
        let trigger = Trigger::parse(key.trim())?;
        all_known.then_some(Self { chord, trigger })
    }

    pub fn text(&self) -> String {
        let mut text = String::new();
        for (name, on) in MODIFIERS.iter().zip(self.chord.flags()) {
            if on {
                text.push_str(name);
                text.push('+');
            }
        }
        text.push_str(&self.trigger.text());
        text
    }
}

impl Trigger {
    fn parse(key: &str) -> Option<Self> {
        let mut chars = key.chars();
        if let (Some(single), None) = (chars.next(), chars.next()) {
            return Some(match single.to_digit(10) {
                Some(digit) => Trigger::Digit(digit as u8),
                None => Trigger::Char(lower(single)?),
            });
        }
        let named = NAMED
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(key));
        let numpad = NUMPAD
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(key));
        named
            .map(|(named, _)| Trigger::Named(*named))
            .or_else(|| numpad.map(|(code, _)| Trigger::Numpad(*code)))
    }

    pub fn text(&self) -> String {
        match self {
            Trigger::Char(single) => single.to_uppercase().collect(),
            Trigger::Digit(digit) => digit.to_string(),
            Trigger::Numpad(code) => name_of(&NUMPAD, code).to_string(),
            Trigger::Named(named) => name_of(&NAMED, named).to_string(),
        }
    }
}

fn split(text: &str) -> (&str, String) {
    match text.strip_suffix('+') {
        Some(body) => match body.rsplit_once('+') {
            Some((mods, key)) => (mods, format!("{key}+")),
            None => ("", format!("{body}+")),
        },
        None => {
            let (mods, key) = text.rsplit_once('+').unwrap_or(("", text));
            (mods, key.to_string())
        }
    }
}

pub(super) fn lower(single: char) -> Option<char> {
    let mut lowered = single.to_lowercase();
    match (lowered.next(), lowered.next()) {
        (Some(only), None) if !only.is_control() && !only.is_whitespace() => Some(only),
        _ => None,
    }
}

fn name_of<T: PartialEq>(table: &[(T, &'static str)], value: &T) -> &'static str {
    table
        .iter()
        .find(|(entry, _)| entry == value)
        .map_or("?", |(_, name)| name)
}
