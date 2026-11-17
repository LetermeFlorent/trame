use trame::log_error;

use super::Running;

impl Running {
    pub(in crate::app::running) fn copy_selection(&self) {
        let session = self.desk.sessions.active(&self.desk.workspace);
        let text = session.and_then(|session| session.terminal().screen().selected_text());
        if let Some(text) = text {
            write_clipboard(&text);
        }
    }
}

pub(in crate::app::running) fn write_clipboard(text: &str) {
    let written = arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text));
    if let Err(error) = written {
        log_error!("the clipboard could not be written: {error}");
    }
}
