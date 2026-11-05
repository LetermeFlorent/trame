use std::collections::HashMap;

use super::decode::{decode_base64, is_invisible_format};

const MAX_RECORDS: usize = 256;
const MAX_ID: usize = 128;
const MAX_SEGMENT: usize = 32;
const MAX_DEPTH: usize = 8;
const MAX_TEXT: usize = 2048;
pub(crate) const PROBE_REPLY: &str =
    "\x1b]7501;?:states=idle,working,done,blocked,error:kinds=permission,question,auth";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Working,
    Done,
    Blocked,
    Error,
}

enum Report {
    Set(String, State, String),
    Clear(String),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ProgramStatus {
    records: HashMap<String, (State, u64)>,
    clock: u64,
}

impl ProgramStatus {
    pub(crate) fn working(&self) -> bool {
        self.records
            .values()
            .any(|(state, _)| *state == State::Working)
    }

    pub(crate) fn apply(&mut self, payload: &[u8]) -> Option<String> {
        match parse(payload)? {
            Report::Clear(id) => {
                let below = format!("{id}/");
                self.records
                    .retain(|key, _| !id.is_empty() && *key != id && !key.starts_with(&below));
                None
            }
            Report::Set(id, state, message) => {
                self.clock += 1;
                if self.records.len() >= MAX_RECORDS && !self.records.contains_key(&id) {
                    self.evict_oldest();
                }
                let before = self.records.insert(id, (state, self.clock));
                let calls = matches!(state, State::Done | State::Blocked | State::Error);
                let changed = before.is_none_or(|(old, _)| old != state);
                (calls && changed).then_some(message)
            }
        }
    }

    pub(crate) fn drop_running(&mut self) {
        self.records
            .retain(|_, (state, _)| !matches!(state, State::Working | State::Blocked));
    }

    fn evict_oldest(&mut self) {
        let oldest = self
            .records
            .iter()
            .min_by_key(|(_, (_, at))| *at)
            .map(|(key, _)| key.clone());
        if let Some(key) = oldest {
            self.records.remove(&key);
        }
    }
}

fn parse(payload: &[u8]) -> Option<Report> {
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
