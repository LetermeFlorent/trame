use std::num::IntErrorKind;

use super::Preferences;

pub const MIN_HISTORY_LINES: usize = 100;
pub const MAX_HISTORY_LINES: usize = 10_000;
pub const DEFAULT_HISTORY_LINES: usize = 1_000;
const HISTORY_CHOICES: [usize; 6] = [100, 500, 1_000, 2_000, 5_000, 10_000];

impl Preferences {
    pub fn step_history_lines(&self, steps: i32) -> Self {
        let mut lines = self.history_lines;
        for _ in 0..steps.unsigned_abs() {
            lines = next_choice(lines, steps > 0);
        }
        self.with_history_lines(lines)
    }

    pub fn with_history_lines(&self, lines: usize) -> Self {
        self.with(|next| next.history_lines = sanitized_history(lines))
    }

    pub fn typed_history_lines(&self, typed: &str) -> Self {
        match typed.trim().parse::<usize>() {
            Ok(lines) => self.with_history_lines(lines),
            Err(error) if *error.kind() == IntErrorKind::PosOverflow => {
                self.with_history_lines(MAX_HISTORY_LINES)
            }
            Err(_) => self.clone(),
        }
    }
}

pub(super) fn sanitized_history(lines: usize) -> usize {
    lines.clamp(MIN_HISTORY_LINES, MAX_HISTORY_LINES)
}

pub(super) fn history_lines() -> usize {
    DEFAULT_HISTORY_LINES
}

fn next_choice(lines: usize, up: bool) -> usize {
    let found = match up {
        true => HISTORY_CHOICES.iter().find(|&&choice| choice > lines),
        false => HISTORY_CHOICES.iter().rev().find(|&&choice| choice < lines),
    };
    found.copied().unwrap_or(lines)
}
