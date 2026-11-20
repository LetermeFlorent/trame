use trame::input::{Command, TextField};
use trame::ui::frame::SettingControl;
use winit::keyboard::{Key, NamedKey};

use super::clipboard::read_plain_clipboard;
use super::keys::DecodedKey;
use crate::gnome::state::WindowState;
use crate::gnome::state::colors::edit_letters;
use crate::gnome::state::editing::{EditMode, Editing};

pub fn editing_key(state: &mut WindowState, decoded: &DecodedKey) -> bool {
    let Some(typing) = typing_mode(state) else {
        return false;
    };
    state.modifiers = decoded.modifiers;
    if !typing {
        if decoded.is_escape() {
            state.step_back();
        } else if decoded.is_enter() {
            state.commit_edit();
        }
        state.changed();
        return true;
    }
    apply_typed_key(state, decoded);
    state.changed();
    true
}

fn typing_mode(state: &WindowState) -> Option<bool> {
    if state.setting_field.is_some() || state.settings_form_open() {
        return Some(true);
    }
    let editing = state.editing.as_ref()?;
    Some(matches!(
        editing.mode,
        EditMode::Text { .. }
            | EditMode::Form(..)
            | EditMode::Picker(..)
            | EditMode::Emblem { .. }
    ))
}

fn apply_typed_key(state: &mut WindowState, decoded: &DecodedKey) {
    if let Key::Named(named) = &decoded.key
        && edit_named(state, *named)
    {
        return;
    }
    if state.keymap().command_for(decoded.shortcut_input()) == Some(Command::Paste) {
        if let Some(text) = read_plain_clipboard() {
            type_into_field(state, &text);
        }
        return;
    }
    let m = decoded.modifiers;
    let plain = !m.super_key() && (!m.control_key() || m.alt_key());
    if let Some(t) = decoded.text.as_deref().filter(|_| plain) {
        type_into_field(state, t);
    }
}

fn edit_named(state: &mut WindowState, key: NamedKey) -> bool {
    match key {
        NamedKey::Enter => state.commit_edit(),
        NamedKey::Tab => {
            let back = state.modifiers.shift_key();
            let Some(form) = state.connect_form_mut() else {
                return false;
            };
            if back { form.previous() } else { form.next() }
        }
        NamedKey::Escape => state.step_back(),
        NamedKey::Backspace => with_field(state, TextField::backspace),
        NamedKey::Delete => with_field(state, TextField::delete),
        NamedKey::ArrowLeft => with_field(state, TextField::left),
        NamedKey::ArrowRight => with_field(state, TextField::right),
        NamedKey::Home => with_field(state, TextField::home),
        NamedKey::End => with_field(state, TextField::end),
        _ => return false,
    }
    true
}

pub fn type_into_field(state: &mut WindowState, typed: &str) {
    let digits = matches!(state.setting_field, Some((SettingControl::HistoryLines, _)));
    let kept: String = typed
        .chars()
        .filter(|c| !digits || c.is_ascii_digit())
        .collect();
    if !kept.is_empty() {
        with_field(state, |f| f.insert(&kept));
    }
}

fn with_field(state: &mut WindowState, change: impl FnOnce(&mut TextField)) {
    if let Some((_, field)) = &mut state.setting_field {
        return change(field);
    }
    if let Some(Editing {
        mode: EditMode::Text { field, .. },
        ..
    }) = &mut state.editing
    {
        return change(field);
    }
    if let Some(picker) = state.editing.as_mut().and_then(Editing::picker_mut) {
        return picker.edit_code(change);
    }
    if let Some(letters) = state.editing.as_mut().and_then(Editing::letters_mut) {
        return edit_letters(letters, change);
    }
    if let Some(form) = state.connect_form_mut() {
        change(form.focused_mut());
    }
}
