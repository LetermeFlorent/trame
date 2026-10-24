mod bindings;

use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey};

use crate::input::Command;
use crate::ui::Platform;
use crate::workspace::Edge;
use bindings::{Bindings, CONTROL, CONTROL_SHIFT, Chord, DIGITS, LETTERS, SHIFT};

#[derive(Clone, Copy, Debug)]
pub struct ShortcutInput<'a> {
    pub key: &'a Key,
    pub base: &'a Key,
    pub physical: PhysicalKey,
    pub modifiers: ModifiersState,
}

pub fn shortcut(input: ShortcutInput, platform: Platform) -> Option<Command> {
    let pressed = Chord {
        control: input.modifiers.control_key(),
        shift: input.modifiers.shift_key(),
        alt: input.modifiers.alt_key(),
        logo: input.modifiers.super_key(),
    };
    let bindings = Bindings::for_platform(platform);
    navigation(input.key, pressed)
        .or_else(|| letter(input.base, pressed, &bindings))
        .or_else(|| digit(input.physical, pressed, &bindings))
        .or_else(|| font(input, pressed, &bindings))
        .or_else(|| focus(input.key, pressed, &bindings))
}

fn navigation(key: &Key, pressed: Chord) -> Option<Command> {
    let Key::Named(named) = key else {
        return None;
    };
    Some(match (named, pressed) {
        (NamedKey::Tab, CONTROL) | (NamedKey::PageDown, CONTROL) => Command::NextTab,
        (NamedKey::Tab, CONTROL_SHIFT) | (NamedKey::PageUp, CONTROL) => Command::PreviousTab,
        (NamedKey::PageDown, CONTROL_SHIFT) => Command::NextInstance,
        (NamedKey::PageUp, CONTROL_SHIFT) => Command::PreviousInstance,
        (NamedKey::PageUp, SHIFT) => Command::ScrollPageUp,
        (NamedKey::PageDown, SHIFT) => Command::ScrollPageDown,
        (NamedKey::Home, CONTROL_SHIFT) => Command::ScrollTop,
        (NamedKey::End, CONTROL_SHIFT) => Command::ScrollBottom,
        (NamedKey::Insert, SHIFT) => Command::Paste,
        _ => return None,
    })
}

fn letter(base: &Key, pressed: Chord, bindings: &Bindings) -> Option<Command> {
    let Key::Character(text) = base else {
        return None;
    };
    let text = text.to_lowercase();
    if pressed == bindings.split_down && text == "d" {
        return Some(Command::SplitDown);
    }
    if Some(pressed) == bindings.clipboard {
        return match text.as_str() {
            "c" => Some(Command::Copy),
            "v" => Some(Command::Paste),
            _ => None,
        };
    }
    if pressed != bindings.primary {
        return None;
    }
    LETTERS
        .iter()
        .find(|(letter, _)| *letter == text)
        .map(|(_, command)| *command)
}

fn digit(physical: PhysicalKey, pressed: Chord, bindings: &Bindings) -> Option<Command> {
    let PhysicalKey::Code(code) = physical else {
        return None;
    };
    if pressed != bindings.select_tab {
        return None;
    }
    match DIGITS.iter().position(|digit| *digit == code)? {
        8 => Some(Command::LastTab),
        index => Some(Command::SelectTab(index)),
    }
}

fn font(input: ShortcutInput, pressed: Chord, bindings: &Bindings) -> Option<Command> {
    let without_shift = Chord {
        shift: false,
        ..pressed
    };
    if without_shift != bindings.font {
        return None;
    }
    let is = |text: &str| {
        [input.key, input.base]
            .iter()
            .any(|key| matches!(key, Key::Character(value) if value == text))
    };
    let code = match input.physical {
        PhysicalKey::Code(code) => Some(code),
        PhysicalKey::Unidentified(_) => None,
    };
    match code {
        Some(KeyCode::NumpadAdd) => return Some(Command::FontBigger),
        Some(KeyCode::NumpadSubtract) => return Some(Command::FontSmaller),
        Some(KeyCode::Digit0 | KeyCode::Numpad0) => return Some(Command::FontReset),
        _ => {}
    }
    match () {
        _ if is("=") || is("+") => Some(Command::FontBigger),
        _ if is("-") => Some(Command::FontSmaller),
        _ => None,
    }
}

fn focus(key: &Key, pressed: Chord, bindings: &Bindings) -> Option<Command> {
    if pressed != bindings.focus_pane {
        return None;
    }
    let Key::Named(named) = key else {
        return None;
    };
    let edge = match named {
        NamedKey::ArrowLeft => Edge::Left,
        NamedKey::ArrowRight => Edge::Right,
        NamedKey::ArrowUp => Edge::Top,
        NamedKey::ArrowDown => Edge::Bottom,
        _ => return None,
    };
    Some(Command::FocusPane(edge))
}
