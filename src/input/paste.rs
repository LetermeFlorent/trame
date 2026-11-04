use crate::term::Modes;

pub const PREVIEW_CHARS: usize = 40;

const PASTE_START: &str = "\x1b[200~";
const PASTE_END: &str = "\x1b[201~";
const LINE_BREAKS: [char; 2] = ['\n', '\r'];
const PREVIEW_LINES: usize = 3;
const CUT_MARK: char = '\u{2026}';

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PastePreview {
    pub lines: Vec<String>,
    pub line_count: usize,
    pub hidden_characters: bool,
}

pub fn paste_bytes(text: &str, modes: Modes) -> Vec<u8> {
    let normalized: String = text
        .replace("\r\n", "\r")
        .replace('\n', "\r")
        .chars()
        .filter(|ch| is_kept(*ch))
        .collect();
    if !modes.contains(Modes::BRACKETED_PASTE) {
        return normalized.into_bytes();
    }
    format!("{PASTE_START}{normalized}{PASTE_END}").into_bytes()
}

pub fn asks_before_pasting(text: &str, modes: Modes) -> bool {
    let runs_lines = !modes.contains(Modes::BRACKETED_PASTE) && text.contains(LINE_BREAKS);
    runs_lines || has_hidden_characters(text)
}

pub fn paste_preview(text: &str) -> PastePreview {
    let mut rows = rows(text.trim_end_matches(LINE_BREAKS));
    let lines: Vec<String> = rows
        .by_ref()
        .take(PREVIEW_LINES)
        .map(preview_line)
        .collect();
    let line_count = lines.len() + rows.count();
    PastePreview {
        lines,
        line_count,
        hidden_characters: has_hidden_characters(text),
    }
}

fn rows(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n')
        .flat_map(|line| line.strip_suffix('\r').unwrap_or(line).split('\r'))
}

fn preview_line(line: &str) -> String {
    let mut shown = line
        .chars()
        .map(|ch| if ch == '\t' { ' ' } else { ch })
        .filter(|ch| !ch.is_control());
    let kept: String = shown.by_ref().take(PREVIEW_CHARS).collect();
    if shown.next().is_some() {
        format!("{kept}{CUT_MARK}")
    } else {
        kept
    }
}

fn has_hidden_characters(text: &str) -> bool {
    text.chars()
        .any(|ch| !is_kept(ch) && !LINE_BREAKS.contains(&ch))
}

fn is_kept(ch: char) -> bool {
    !ch.is_control() || matches!(ch, '\t' | '\r')
}
