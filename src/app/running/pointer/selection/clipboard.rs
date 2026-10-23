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
}

fn write_clipboard(text: &str) {
    let written = arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text));
    if let Err(error) = written {
        eprintln!("trame: the clipboard could not be written: {error}");
    }
}
