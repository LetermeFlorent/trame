use trame::session::SessionId;

use super::Running;

impl Running {
    pub(in crate::app::running) fn copy_selection(&self) {
        let session = self.sessions.active(&self.workspace);
        let text = session.and_then(|session| session.terminal().screen().selected_text());
        if let Some(text) = text {
            write_clipboard(&text);
        }
    }

    pub(in crate::app::running) fn forward_clipboard(&self, id: SessionId) {
        let session = self.sessions.get(id);
        let text = session.and_then(|session| session.terminal().screen_mut().take_clipboard());
        if let Some(text) = text {
            write_clipboard(&text);
        }
    }

    pub(in crate::app::running) fn forward_attention(&mut self, id: SessionId) {
        let Some(session) = self.sessions.get(id) else {
            return;
        };
        let Some(message) = session.terminal().screen_mut().take_attention() else {
            return;
        };
        let seen = self.view.window_focused && self.pane_showing(id).is_some();
        if seen || !self.preferences.notify_waiting {
            return;
        }
        let body = match message.is_empty() {
            true => self.strings.waiting_notice,
            false => &message,
        };
        let title = session.label();
        self.notifier.show(&self.window, &title, body);
    }
}

fn write_clipboard(text: &str) {
    let written = arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text));
    if let Err(error) = written {
        eprintln!("trame: the clipboard could not be written: {error}");
    }
}
