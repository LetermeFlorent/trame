use crate::term::Modes;

const PASTE_START: &str = "\x1b[200~";
const PASTE_END: &str = "\x1b[201~";
const ESC: char = '\x1b';

pub fn paste_bytes(text: &str, modes: Modes) -> Vec<u8> {
    let normalized = text.replace("\r\n", "\r").replace('\n', "\r");
    if !modes.contains(Modes::BRACKETED_PASTE) {
        return normalized.into_bytes();
    }
    let body: String = normalized.chars().filter(|ch| *ch != ESC).collect();
    format!("{PASTE_START}{body}{PASTE_END}").into_bytes()
}
