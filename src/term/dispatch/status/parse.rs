use std::collections::HashMap;

use super::State;
use crate::term::dispatch::decode::{decode_base64, is_invisible_format};

const MAX_ID: usize = 128;
const MAX_SEGMENT: usize = 32;
const MAX_DEPTH: usize = 8;
const MAX_TEXT: usize = 2048;

pub(super) enum Report {
    Set(String, State, String),
    Clear(String),
}

pub(super) fn parse(payload: &[u8]) -> Option<Report> {
    let text = std::str::from_utf8(payload).ok()?;
    let mut fields: HashMap<&str, &str> = HashMap::new();
    for pair in text.split(':') {
        if let Some((key, value)) = pair.split_once('=') {
            fields.insert(key, value);
        }
    }
    let id = fields.get("id").copied().unwrap_or_default();
    if !valid_id(id) {
        return None;
    }
    let message = match fields.get("msg") {
        Some(encoded) => decoded(encoded)?,
        None => String::new(),
    };
    if let Some(title) = fields.get("title") {
        decoded(title)?;
    }
    let state = match *fields.get("state")? {
        "idle" => State::Idle,
        "working" => State::Working,
        "done" => State::Done,
        "blocked" => State::Blocked,
        "error" => State::Error,
        "clear" => return Some(Report::Clear(id.to_string())),
        _ => return None,
    };
    Some(Report::Set(id.to_string(), state, message))
}

fn valid_id(id: &str) -> bool {
    id.is_empty()
        || (id.len() <= MAX_ID
            && id.split('/').count() <= MAX_DEPTH
            && id.split('/').all(|segment| {
                (1..=MAX_SEGMENT).contains(&segment.len())
                    && segment
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_.+-".contains(&b))
            }))
}

fn decoded(encoded: &str) -> Option<String> {
    let text = String::from_utf8(decode_base64(encoded.as_bytes())?).ok()?;
    if text.len() > MAX_TEXT || text.chars().any(char::is_control) {
        return None;
    }
    Some(text.chars().filter(|&c| !is_invisible_format(c)).collect())
}
