use trame::files::{join_paths, save_capture};
use trame::log_error;

use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn paste(&mut self) {
        let Some(text) = read_terminal_clipboard() else {
            return;
        };
        let effects = self.desk.paste(text);
        self.apply(effects);
    }
}

pub(in crate::app::running) fn read_clipboard() -> Option<String> {
    clipboard_part(arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()))
}

pub(in crate::app::running) fn read_terminal_clipboard() -> Option<String> {
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(error) => {
            log_error!("the clipboard could not be opened: {error}");
            return None;
        }
    };
    if let Some(paths) = clipboard_part(clipboard.get().file_list()).filter(|p| !p.is_empty()) {
        return Some(join_paths(&paths));
    }
    if let Some(text) = clipboard_part(clipboard.get_text()) {
        return Some(text);
    }
    let image = clipboard_part(clipboard.get_image())?;
    match save_capture(image.width, image.height, &image.bytes) {
        Ok(path) => Some(path),
        Err(error) => {
            log_error!("the copied image could not be saved: {error}");
            None
        }
    }
}

fn clipboard_part<T>(read: Result<T, arboard::Error>) -> Option<T> {
    match read {
        Ok(value) => Some(value),
        Err(arboard::Error::ContentNotAvailable) => None,
        Err(error) => {
            log_error!("the clipboard could not be read: {error}");
            None
        }
    }
}
