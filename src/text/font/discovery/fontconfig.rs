use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use super::{FontSource, SystemFonts, is_file};

const TERMINAL: &[&str] = &[
    "monospace",
    "monospace:bold",
    "monospace:italic",
    "monospace:bold:italic",
];

const INTERFACE: &[&str] = &["sans-serif", "sans-serif:bold", "sans-serif:italic"];

const FORMAT: &str = "--format=%{file}|%{index}";

pub fn system_fonts() -> SystemFonts {
    let children: Vec<Option<Child>> = TERMINAL
        .iter()
        .chain(INTERFACE)
        .map(|pattern| spawn_match(pattern))
        .collect();
    let mut matches: Vec<Option<FontSource>> = children
        .into_iter()
        .map(|child| child.and_then(finish_match))
        .collect();
    let interface = matches.split_off(TERMINAL.len());
    SystemFonts {
        terminal: unique(matches),
        interface: unique(interface),
        emoji: Vec::new(),
        fallback: Vec::new(),
    }
}

pub fn fallback_for(ch: char, emoji: bool) -> Option<FontSource> {
    let family = if emoji { "emoji" } else { "monospace" };
    let pattern = format!("{family}:charset={:x}", u32::from(ch));
    spawn_match(&pattern).and_then(finish_match)
}

fn spawn_match(pattern: &str) -> Option<Child> {
    Command::new("fc-match")
        .arg(FORMAT)
        .arg(pattern)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn finish_match(child: Child) -> Option<FontSource> {
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_match(&output.stdout)
}

fn parse_match(bytes: &[u8]) -> Option<FontSource> {
    let split = bytes.iter().rposition(|byte| *byte == b'|')?;
    let path = PathBuf::from(OsStr::from_bytes(&bytes[..split]));
    let index = std::str::from_utf8(&bytes[split + 1..])
        .ok()?
        .trim()
        .parse()
        .ok()?;
    is_file(&path).then_some(FontSource {
        path,
        index: Some(index),
    })
}

fn unique(sources: Vec<Option<FontSource>>) -> Vec<FontSource> {
    let mut found: Vec<FontSource> = Vec::new();
    for source in sources.into_iter().flatten() {
        if !found.contains(&source) {
            found.push(source);
        }
    }
    found
}
