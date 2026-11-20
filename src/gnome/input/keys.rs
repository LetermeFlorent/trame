use gtk::gdk;
use trame::input::{Command, KeyInput, ShortcutInput, encode};
use trame::term::{Modes, Terminal};
use winit::keyboard::{Key, ModifiersState, NamedKey, PhysicalKey, SmolStr};

use super::keycodes::{named_key, physical_of};

pub struct DecodedKey {
    pub key: Key,
    pub base: Key,
    pub physical: PhysicalKey,
    pub modifiers: ModifiersState,
    pub text: Option<String>,
}

impl DecodedKey {
    pub fn from_gdk(keyval: gdk::Key, keycode: u32, mask: gdk::ModifierType) -> Option<Self> {
        let modifiers = gdk_modifiers(mask);
        let physical = physical_of(keyval, keycode);
        let key = logical_key(keyval)?;
        let base = logical_key(keyval.to_lower()).unwrap_or_else(|| key.clone());
        let text = keyval
            .to_unicode()
            .filter(|ch| !ch.is_control())
            .map(String::from);
        Some(Self {
            key,
            base,
            physical,
            modifiers,
            text,
        })
    }

    pub fn shortcut_input(&self) -> ShortcutInput<'_> {
        ShortcutInput {
            key: &self.key,
            base: &self.base,
            physical: self.physical,
            modifiers: self.modifiers,
        }
    }

    pub fn bytes(&self, modes: Modes) -> Option<Vec<u8>> {
        let input = KeyInput {
            key: &self.key,
            base: &self.base,
            text: self.text.as_deref(),
            modifiers: self.modifiers,
        };
        encode(input, modes)
    }

    pub fn is_escape(&self) -> bool {
        self.key == Key::Named(NamedKey::Escape)
    }

    pub fn is_enter(&self) -> bool {
        self.key == Key::Named(NamedKey::Enter)
    }
}

pub fn takes_key(command: Command, modifiers: ModifiersState, terminal: &Terminal) -> bool {
    command != Command::Copy
        || modifiers != ModifiersState::CONTROL
        || terminal.screen().has_selected_text()
}

pub fn after_copy(terminal: &mut Terminal, modifiers: ModifiersState) {
    if modifiers == ModifiersState::CONTROL {
        terminal.screen_mut().clear_selection();
    }
}

fn logical_key(keyval: gdk::Key) -> Option<Key> {
    if let Some(named) = named_key(keyval) {
        return Some(Key::Named(named));
    }
    let ch = keyval.to_unicode().filter(|ch| !ch.is_control())?;
    Some(Key::Character(SmolStr::new(ch.to_string())))
}

pub fn gdk_modifiers(mask: gdk::ModifierType) -> ModifiersState {
    let pairs = [
        (gdk::ModifierType::SHIFT_MASK, ModifiersState::SHIFT),
        (gdk::ModifierType::CONTROL_MASK, ModifiersState::CONTROL),
        (gdk::ModifierType::ALT_MASK, ModifiersState::ALT),
        (gdk::ModifierType::SUPER_MASK, ModifiersState::SUPER),
    ];
    pairs
        .into_iter()
        .filter(|(bit, _)| mask.contains(*bit))
        .fold(ModifiersState::empty(), |all, (_, one)| all | one)
}
