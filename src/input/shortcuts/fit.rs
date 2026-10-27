use winit::keyboard::{Key, ModifiersState, PhysicalKey};

use super::ShortcutInput;
use super::combo::{Chord, Combo, Trigger, lower};
use super::names::DIGITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Fit {
    Exact,
    Loose,
}

impl Combo {
    pub(super) fn fit(&self, input: &ShortcutInput) -> Option<Fit> {
        if alt_graph(input) {
            return None;
        }
        let pressed = chord_of(input.modifiers);
        let fit = if pressed == self.chord {
            Fit::Exact
        } else if self.loosely(pressed) {
            Fit::Loose
        } else {
            return None;
        };
        self.trigger.hit(input).then_some(fit)
    }

    fn loosely(&self, pressed: Chord) -> bool {
        let unshifted = Chord {
            shift: false,
            ..pressed
        };
        pressed.shift
            && !self.chord.shift
            && unshifted == self.chord
            && self.trigger.ignores_shift()
    }
}

impl Trigger {
    fn ignores_shift(self) -> bool {
        match self {
            Trigger::Char(single) => !single.is_alphabetic(),
            Trigger::Digit(_) | Trigger::Numpad(_) => true,
            Trigger::Named(_) => false,
        }
    }

    fn hit(self, input: &ShortcutInput) -> bool {
        match (self, input.physical) {
            (Trigger::Named(named), _) => *input.key == Key::Named(named),
            (Trigger::Digit(digit), PhysicalKey::Code(code)) => DIGITS[usize::from(digit)] == code,
            (Trigger::Digit(digit), PhysicalKey::Unidentified(_)) => {
                char_of(input.base) == char::from_digit(u32::from(digit), 10)
            }
            (Trigger::Numpad(numpad), physical) => physical == PhysicalKey::Code(numpad),
            (Trigger::Char(single), _) => {
                char_of(input.base) == Some(single)
                    || (!single.is_alphabetic() && char_of(input.key) == Some(single))
            }
        }
    }
}

pub(super) fn chord_of(modifiers: ModifiersState) -> Chord {
    Chord::new(
        modifiers.control_key(),
        modifiers.shift_key(),
        modifiers.alt_key(),
        modifiers.super_key(),
    )
}

pub(super) fn char_of(key: &Key) -> Option<char> {
    let Key::Character(text) = key else {
        return None;
    };
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(only), None) => lower(only),
        _ => None,
    }
}

pub(super) fn alt_graph(input: &ShortcutInput) -> bool {
    let held = input.modifiers.control_key() && input.modifiers.alt_key();
    let printed = (char_of(input.key), char_of(input.base));
    held && matches!(printed, (Some(key), Some(base)) if key != base)
}
