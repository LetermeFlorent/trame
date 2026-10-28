use std::rc::Rc;

use trame::input::{asks_before_pasting, paste_bytes, paste_preview};
use trame::session::{Session, SessionId};

use crate::app::running::Running;
use crate::app::running::tabs::Closing;

pub(in crate::app::running) struct PendingPaste {
    session: SessionId,
    text: String,
    pub(in crate::app::running) body: String,
}

impl Running {
    pub(in crate::app::running) fn paste(&mut self) {
        let Some(text) = read_clipboard() else {
            return;
        };
        let Some(session) = self.sessions.active(&self.workspace) else {
            return;
        };
        let id = session.id();
        let modes = session.terminal().screen().modes();
        if !self.preferences.confirm_paste || !asks_before_pasting(&text, modes) {
            return self.write_to_active(|modes| paste_bytes(&text, modes));
        }
        if self.closing.is_none() {
            let body = self.paste_body(&text);
            let pending = PendingPaste {
                session: id,
                text,
                body,
            };
            self.closing = Some(Closing::Paste(Rc::new(pending)));
            self.changed();
        }
    }

    pub(in crate::app::running) fn send_paste(&mut self, pending: &PendingPaste) {
        let active = self.sessions.active(&self.workspace).map(Session::id);
        if active == Some(pending.session) {
            self.write_to_active(|modes| paste_bytes(&pending.text, modes));
        }
    }

    fn paste_body(&self, text: &str) -> String {
        let strings = self.strings;
        let preview = paste_preview(text);
        let mut lines = preview.lines;
        if preview.line_count > lines.len() {
            lines.push(format!("{} {}", preview.line_count, strings.lines_in_all));
        }
        if preview.hidden_characters {
            lines.push(strings.hidden_removed.to_owned());
        }
        lines.join("\n")
    }
}

pub(in crate::app::running) fn read_clipboard() -> Option<String> {
    match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
        Ok(text) => Some(text),
        Err(arboard::Error::ContentNotAvailable) => None,
        Err(error) => {
            eprintln!("trame: the clipboard could not be read: {error}");
            None
        }
    }
}
