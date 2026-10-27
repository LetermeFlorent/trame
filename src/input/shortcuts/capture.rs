use winit::keyboard::{Key, PhysicalKey};

use super::ShortcutInput;
use super::combo::{Chord, Combo, Trigger};
use super::fit::{alt_graph, char_of, chord_of};
use super::names::{DIGITS, MODIFIER_KEYS, NAMED, NUMPAD, is_function_key};
use crate::ui::Platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressed {
    Modifier,
    Unsupported,
    Typing,
    Combo(Combo),
}

pub fn pressed(input: ShortcutInput, platform: Platform) -> Pressed {
    if let Key::Named(named) = input.key
        && MODIFIER_KEYS.contains(named)
    {
        return Pressed::Modifier;
    }
    if alt_graph(&input) {
        return Pressed::Typing;
    }
    let Some(trigger) = trigger_of(&input) else {
        return Pressed::Unsupported;
    };
    let chord = chord_of(input.modifiers);
    if types(chord, trigger, platform) {
        return Pressed::Typing;
    }
    Pressed::Combo(Combo::new(chord, trigger))
}

fn trigger_of(input: &ShortcutInput) -> Option<Trigger> {
    if let Key::Named(named) = input.key {
        let known = NAMED.iter().any(|(key, _)| key == named);
        return known.then_some(Trigger::Named(*named));
    }
    if let PhysicalKey::Code(code) = input.physical {
        if let Some(digit) = DIGITS.iter().position(|key| *key == code) {
            return Some(Trigger::Digit(digit as u8));
        }
        if NUMPAD.iter().any(|(key, _)| *key == code) {
            return Some(Trigger::Numpad(code));
        }
    }
    let single = char_of(input.base)?;
    Some(match single.to_digit(10) {
        Some(digit) => Trigger::Digit(digit as u8),
        None => Trigger::Char(single),
    })
}

fn types(chord: Chord, trigger: Trigger, platform: Platform) -> bool {
    let command = chord.control || chord.logo;
    match trigger {
        Trigger::Named(named) => !command && !chord.alt && !chord.shift && !is_function_key(named),
        _ => !command && (!chord.alt || platform == Platform::Mac),
    }
}
