use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

use trame::input::{Command, KeyInput, Keymap, ShortcutInput, encode};
use trame::term::Modes;

use super::copy::{after_copy, takes_key};
use crate::app::running::Running;
use crate::app::window;

const FOCUS_IN: &[u8] = b"\x1b[I";
const FOCUS_OUT: &[u8] = b"\x1b[O";

impl Running {
    pub(in crate::app::running) fn key(&mut self, exiting: bool, event: &KeyEvent) {
        if self.frame.blocked
            || event.state != ElementState::Pressed
            || self.closing_key(event)
            || self.drag_key(&event.logical_key)
            || self.menu_key(event)
            || self.editing_key(event)
            || self.settings_key(event)
        {
            return;
        }
        if let Some(command) = self
            .shortcut_for(event)
            .filter(|command| self.claims(*command))
        {
            self.command(exiting, command);
            return;
        }
        let base = event.key_without_modifiers();
        let key_input = KeyInput {
            key: &event.logical_key,
            base: &base,
            text: event.text_with_all_modifiers(),
            modifiers: self.modifiers,
        };
        self.write_to_active(|modes| encode(key_input, modes).unwrap_or_default());
    }

    pub(in crate::app::running) fn shortcut_for(&mut self, event: &KeyEvent) -> Option<Command> {
        let base = event.key_without_modifiers();
        let input = shortcut_input(event, &base, self.modifiers);
        self.keymap().command_for(input)
    }

    pub(in crate::app::running) fn keymap(&mut self) -> &Keymap {
        let platform = self.metrics.platform;
        self.keymap
            .get(&self.desk.preferences.shortcuts, |overrides| {
                Keymap::new(platform, overrides)
            })
    }

    fn claims(&self, command: Command) -> bool {
        self.desk
            .sessions
            .active(&self.desk.workspace)
            .is_none_or(|session| takes_key(command, self.modifiers, &session.terminal()))
    }

    pub(in crate::app::running) fn copy_shortcut(&mut self) {
        self.copy_selection();
        if let Some(session) = self.desk.sessions.active(&self.desk.workspace) {
            after_copy(&mut session.terminal(), self.modifiers);
        }
    }

    pub(in crate::app::running) fn write_to_active(
        &mut self,
        bytes_for: impl FnOnce(Modes) -> Vec<u8>,
    ) {
        let effects = self.desk.write_to_active(bytes_for);
        self.apply(effects);
    }

    pub(in crate::app::running) fn focus_changed(&mut self, focused: bool) {
        self.view.window_focused = focused;
        if focused {
            self.notifier.clear();
            self.set_appearance(window::appearance(&self.window));
        } else {
            self.cancel_drag();
            self.modifiers = ModifiersState::empty();
            self.drop_press();
        }
        if let Some(session) = self.desk.sessions.active(&self.desk.workspace) {
            let reports = session
                .terminal()
                .screen()
                .modes()
                .contains(Modes::FOCUS_EVENTS);
            if reports {
                session.write(if focused { FOCUS_IN } else { FOCUS_OUT });
            }
        }
        self.window.request_redraw();
    }
}

pub(in crate::app::running) fn shortcut_input<'a>(
    event: &'a KeyEvent,
    base: &'a Key,
    modifiers: ModifiersState,
) -> ShortcutInput<'a> {
    ShortcutInput {
        key: &event.logical_key,
        base,
        physical: event.physical_key,
        modifiers,
    }
}

pub(in crate::app::running) fn is_escape(event: &KeyEvent) -> bool {
    event.logical_key == Key::Named(NamedKey::Escape)
}
