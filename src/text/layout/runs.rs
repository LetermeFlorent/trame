use std::ops::Range;

use swash::text::{Codepoint, Script};

use super::TextStyle;
use crate::text::font::{FontKey, FontLibrary};

pub(super) struct Run {
    pub(super) range: Range<usize>,
    pub(super) font: FontKey,
    pub(super) script: Option<Script>,
}

pub fn script_of(ch: char) -> Option<Script> {
    Some(ch.script()).filter(|script| !is_neutral(*script))
}

pub(super) fn split_runs(
    library: &mut FontLibrary,
    primary: FontKey,
    text: &str,
    style: TextStyle,
) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for (start, ch) in text.char_indices() {
        let font = font_for(library, primary, ch, style);
        let script = script_of(ch);
        let end = start + ch.len_utf8();
        match runs.last_mut() {
            Some(run)
                if run.font == font
                    && (script.is_none() || run.script.is_none_or(|s| Some(s) == script)) =>
            {
                run.range.end = end;
                run.script = run.script.or(script);
            }
            _ => runs.push(Run {
                range: start..end,
                font,
                script,
            }),
        }
    }
    runs
}

fn font_for(library: &mut FontLibrary, primary: FontKey, ch: char, style: TextStyle) -> FontKey {
    if ch.is_control() {
        return primary;
    }
    library
        .covering_font(primary, ch, style.weight, false)
        .unwrap_or(primary)
}

fn is_neutral(script: Script) -> bool {
    matches!(script, Script::Common | Script::Inherited | Script::Unknown)
}
