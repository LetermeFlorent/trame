use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

use trame::input::{Command, TextField};
use trame::ui::frame::SettingControl;

use super::icons::edit_letters;
use super::{EditMode, Editing, Running};
use crate::app::running::keys::{is_escape, read_clipboard};

impl Running {
    pub(in super::super) fn editing_key(&mut self, event: &KeyEvent) -> bool {
        let typing = match &self.editing {
            _ if self.setting_field.is_some() || self.settings_form_open() => true,
            Some(editing) => matches!(
                editing.mode,
                EditMode::Text { .. }
                    | EditMode::Form(..)
                    | EditMode::Picker(..)
                    | EditMode::Emblem { .. }
            ),
            None => return false,
        };
        match typing {
            true => self.edit_key(event),
            false if is_escape(event) => self.step_back(),
            false if event.logical_key == Key::Named(NamedKey::Enter) => self.commit_edit(),
            false => {}
        }
        self.changed();
        true
    }

    fn edit_key(&mut self, event: &KeyEvent) {
        if let Key::Named(named) = &event.logical_key
            && self.edit_named(*named)
        {
            return;
        }
        if self.is_paste(event) {
            if let Some(text) = read_clipboard() {
                self.type_text(&text);
            }
            return;
        }
        let modifiers = self.modifiers;
        let types_text =
            !modifiers.super_key() && (!modifiers.control_key() || modifiers.alt_key());
        if let Some(typed) = event.text_with_all_modifiers().filter(|_| types_text) {
            self.type_text(typed);
        }
    }

    fn type_text(&mut self, typed: &str) {
        let digits = matches!(self.setting_field, Some((SettingControl::HistoryLines, _)));
        let kept: String = typed
            .chars()
            .filter(|letter| !digits || letter.is_ascii_digit())
            .collect();
        if !kept.is_empty() {
            self.with_field(|field| field.insert(&kept));
        }
    }

    fn edit_named(&mut self, key: NamedKey) -> bool {
        match key {
            NamedKey::Enter => self.commit_edit(),
            NamedKey::Tab => return self.step_form(),
            NamedKey::Escape => self.step_back(),
            NamedKey::Backspace => self.with_field(TextField::backspace),
            NamedKey::Delete => self.with_field(TextField::delete),
            NamedKey::ArrowLeft => self.with_field(TextField::left),
            NamedKey::ArrowRight => self.with_field(TextField::right),
            NamedKey::Home => self.with_field(TextField::home),
            NamedKey::End => self.with_field(TextField::end),
            _ => return false,
        }
        true
    }

    fn step_form(&mut self) -> bool {
        let backwards = self.modifiers.shift_key();
        let Some(form) = self.connect_form_mut() else {
            return false;
        };
        match backwards {
            true => form.previous(),
            false => form.next(),
        }
        true
    }

    fn is_paste(&self, event: &KeyEvent) -> bool {
        self.shortcut_for(event) == Some(Command::Paste)
    }

    fn with_field(&mut self, change: impl FnOnce(&mut TextField)) {
        let text = match (&mut self.setting_field, &mut self.editing) {
            (Some((_, field)), _)
            | (
                None,
                Some(Editing {
                    mode: EditMode::Text { field, .. },
                    ..
                }),
            ) => Some(field),
            _ => None,
        };
        if let Some(field) = text {
            return change(field);
        }
        if let Some(picker) = self.editing.as_mut().and_then(Editing::picker_mut) {
            return picker.edit_code(change);
        }
        if let Some(letters) = self.editing.as_mut().and_then(Editing::letters_mut) {
            return edit_letters(letters, change);
        }
        if let Some(form) = self.connect_form_mut() {
            change(form.focused_mut());
        }
    }
}
