use winit::event::{ElementState, KeyEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

use trame::input::{KeyInput, ShortcutInput, encode, shortcut};
use trame::session::Session;
use trame::term::Modes;

use super::{Running, window};

const FOCUS_IN: &[u8] = b"\x1b[I";
const FOCUS_OUT: &[u8] = b"\x1b[O";

impl Running {
    pub(super) fn key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        if self.frame.blocked
            || event.state != ElementState::Pressed
            || self.drag_key(&event.logical_key)
            || self.menu_key(event)
            || self.editing_key(event)
            || self.settings_key(event)
        {
            return;
        }
        let base = event.key_without_modifiers();
        let shortcut_input = ShortcutInput {
            key: &event.logical_key,
            base: &base,
            physical: event.physical_key,
            modifiers: self.modifiers,
        };
        if let Some(command) = shortcut(shortcut_input, self.metrics.platform) {
            self.command(event_loop, command);
            return;
        }
        let key_input = KeyInput {
            key: &event.logical_key,
            base: &base,
            text: event.text_with_all_modifiers(),
            modifiers: self.modifiers,
        };
        self.write_to_active(|modes| encode(key_input, modes).unwrap_or_default());
    }

    pub(super) fn write_to_active(&mut self, bytes_for: impl FnOnce(Modes) -> Vec<u8>) {
        let Some(session) = self.active_session() else {
            return;
        };
        let id = session.id();
        let bytes = {
            let mut terminal = session.terminal();
            let bytes = bytes_for(terminal.screen().modes());
            if !bytes.is_empty() {
                let screen = terminal.screen_mut();
                screen.reset_display();
                screen.clear_selection();
            }
            bytes
        };

        if !bytes.is_empty() {
            session.write(bytes);
            self.motion.stop_scroll(id);
            self.window.request_redraw();
        }
    }

    pub(super) fn focus_changed(&mut self, focused: bool) {
        self.view.window_focused = focused;
        if focused {
            self.set_appearance(window::appearance(&self.window));
        } else {
            self.cancel_drag();
            self.modifiers = ModifiersState::empty();
            self.pointer.pressed = None;
            self.view.pressed = None;
        }
        if let Some(session) = self.active_session() {
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

    fn active_session(&self) -> Option<&Session> {
        self.workspace
            .active_tab()
            .and_then(|id| self.sessions.get(id))
    }
}

pub(super) fn is_escape(event: &KeyEvent) -> bool {
    event.logical_key == Key::Named(NamedKey::Escape)
}
