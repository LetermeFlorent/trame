use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

use trame::input::{Command, TextField};

use super::{EditMode, Editing, Running};
use crate::app::running::keys::{is_escape, read_clipboard};

impl Running {
    pub(in super::super) fn editing_key(&mut self, event: &KeyEvent) -> bool {
        let typing = match &self.editing {
            _ if self.command_field.is_some() => true,
            Some(editing) => matches!(editing.mode, EditMode::Text { .. } | EditMode::Form(..)),
            None => return false,
        };
        match typing {
            true => self.edit_key(event),
            false if is_escape(event) => self.cancel_edit(),
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
                self.with_field(|field| field.insert(&text));
            }
            return;
        }
        let modifiers = self.modifiers;
        let types_text =
            !modifiers.super_key() && (!modifiers.control_key() || modifiers.alt_key());
        if let Some(typed) = event.text_with_all_modifiers().filter(|_| types_text) {
            self.with_field(|field| field.insert(typed));
        }
    }

    fn edit_named(&mut self, key: NamedKey) -> bool {
        match key {
            NamedKey::Enter => self.commit_edit(),
            NamedKey::Tab => return self.step_form(),
            NamedKey::Escape => self.cancel_edit(),
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
        let Some(Editing {
            mode: EditMode::Form(form, _),
            ..
        }) = &mut self.editing
        else {
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
        let field = match (&mut self.command_field, &mut self.editing) {
            (Some(field), _)
            | (
                None,
                Some(Editing {
                    mode: EditMode::Text { field, .. },
                    ..
                }),
            ) => field,
            (
                None,
                Some(Editing {
                    mode: EditMode::Form(form, _),
                    ..
                }),
            ) => form.focused_mut(),
            _ => return,
        };
        change(field);
    }
}
