mod cursor;
mod target;
#[cfg(test)]
mod tests;
mod text;

use std::path::Path;

use trame::ui::Hit;

use crate::app::running::Running;
use cursor::cursor_in;
use target::{DropTarget, drop_target};
use text::dropped_text;

impl Running {
    pub(in crate::app::running) fn file_dropped(&mut self, path: &Path) {
        let text = dropped_text([path]);
        if text.is_empty() {
            return;
        }
        match drop_target(self.dropped_on(), self.modal_open()) {
            DropTarget::Ignored => return,
            DropTarget::Pane(pane) => self.focus_pane(pane),
            DropTarget::Focused => {}
        }
        self.write_to_active(|_| text.into_bytes());
        if !self.view.window_focused {
            self.window.focus_window();
        }
        self.changed();
    }

    fn dropped_on(&self) -> Option<Hit> {
        cursor_in(&self.window)
            .or(self.pointer.position)
            .and_then(|point| self.frame.hit(point, self.resizable()))
    }

    fn modal_open(&self) -> bool {
        self.frame.blocked
            || self.closing.is_some()
            || self.menu.is_some()
            || self.editing.is_some()
            || self.settings.is_some()
            || self.setting_field.is_some()
    }
}
