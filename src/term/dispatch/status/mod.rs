use std::collections::HashMap;
use std::time::{Duration, Instant};

mod parse;

use parse::{Report, parse};

const MAX_RECORDS: usize = 256;
const QUIET_AFTER: Duration = Duration::from_secs(15);
pub(crate) const PROBE_REPLY: &str =
    "\x1b]7501;?:states=idle,working,done,blocked,error:kinds=permission,question,auth";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum State {
    Idle,
    Working,
    Done,
    Blocked,
    Error,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ProgramStatus {
    records: HashMap<String, (State, u64)>,
    clock: u64,
    command: bool,
    exit: Option<i32>,
    heard: Option<Instant>,
}

impl ProgramStatus {
    pub(crate) fn working(&self, now: Instant) -> bool {
        let reported = self
            .records
            .values()
            .any(|(state, _)| *state == State::Working);
        let recent = self
            .heard
            .is_some_and(|at| now.saturating_duration_since(at) <= QUIET_AFTER);
        recent && (reported || (self.command && self.records.is_empty()))
    }

    pub(crate) fn hear(&mut self, now: Instant) {
        self.heard = Some(now);
    }

    pub(crate) fn start_command(&mut self) {
        self.command = true;
        self.records.clear();
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

    pub(crate) fn end_command(&mut self, code: Option<&[u8]>) {
        if self.command {
            self.exit = code
                .and_then(|code| std::str::from_utf8(code).ok())
                .and_then(|code| code.parse().ok());
        }
        self.drop_running();
    }

    pub(crate) fn take_exit(&mut self) -> Option<i32> {
        self.exit.take()
    }

    pub(crate) fn drop_running(&mut self) {
        self.command = false;
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
