use std::rc::Rc;

use crate::input::{asks_before_pasting, paste_bytes, paste_preview};
use crate::locale::COUNT_SLOT;
use crate::session::{Session, SessionId};

use super::{Closing, Desk, Effect, Effects};

#[derive(Debug)]
pub struct PendingPaste {
    session: SessionId,
    text: String,
    pub body: String,
}

impl Desk {
    pub fn paste(&mut self, text: String) -> Effects {
        let Some(session) = self.sessions.active(&self.workspace) else {
            return Effects::default();
        };
        let id = session.id();
        let modes = session.terminal().screen().modes();
        if !self.preferences.confirm_paste || !asks_before_pasting(&text, modes) {
            return self.write_to_active(|modes| paste_bytes(&text, modes));
        }
        if self.closing.is_some() {
            return Effects::default();
        }
        let body = self.paste_body(&text);
        let pending = PendingPaste {
            session: id,
            text,
            body,
        };
        self.closing = Some(Closing::Paste(Rc::new(pending)));
        Effect::Changed.into()
    }

    pub(super) fn send_paste(&mut self, pending: &PendingPaste) -> Effects {
        let active = self.sessions.active(&self.workspace).map(Session::id);
        if active != Some(pending.session) {
            return Effects::default();
        }
        self.write_to_active(|modes| paste_bytes(&pending.text, modes))
    }

    fn paste_body(&self, text: &str) -> String {
        let strings = self.strings;
        let preview = paste_preview(text);
        let mut lines = preview.lines;
        if preview.line_count > lines.len() {
            let count = preview.line_count.to_string();
            lines.push(strings.lines_in_all.replace(COUNT_SLOT, &count));
        }
        if preview.hidden_characters {
            lines.push(strings.hidden_removed.to_owned());
        }
        lines.join("\n")
    }
}
