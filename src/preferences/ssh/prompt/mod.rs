mod login;

use std::hint::black_box;
use std::time::{Duration, Instant};

pub use login::SshLogin;

pub const ANSWER_WINDOW: Duration = Duration::from_secs(60);

const PASSPHRASE_PROMPT: &str = "enter passphrase for key";
const PASSWORD_PROMPT: &str = "'s password:";
const INTERACTIVE_PROMPT: &str = ") password:";
const HOST_KEY_QUESTION: &str = "(yes/no";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt {
    Password,
    Passphrase,
}

pub fn prompt_in(line: &str) -> Option<Prompt> {
    let line = line.trim_end().to_lowercase();
    if line.starts_with(PASSPHRASE_PROMPT) && line.ends_with(':') {
        return Some(Prompt::Passphrase);
    }
    let interactive = line.starts_with('(') && line.ends_with(INTERACTIVE_PROMPT);
    (line.ends_with(PASSWORD_PROMPT) || interactive).then_some(Prompt::Password)
}

pub struct SecretAnswer {
    secret: String,
    login: SshLogin,
    deadline: Instant,
    started: bool,
    done: bool,
}

impl SecretAnswer {
    pub fn new(secret: String, login: SshLogin, now: Instant) -> Self {
        Self {
            secret,
            login,
            deadline: now + ANSWER_WINDOW,
            started: false,
            done: false,
        }
    }

    pub fn reply(&mut self, line: &str, now: Instant) -> Option<String> {
        if self.is_over(now) {
            return None;
        }
        let line = line.trim().to_lowercase();
        if self.login.is_asked_by(&line) {
            self.done = true;
            return Some(format!("{}\r", self.secret));
        }
        if self.login.is_typed_in(&line) {
            self.started = true;
        } else if self.started && !is_part_of_connecting(&line) {
            self.done = true;
        }
        None
    }

    pub fn is_over(&self, now: Instant) -> bool {
        now >= self.deadline || self.done
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }
}

fn is_part_of_connecting(line: &str) -> bool {
    line.is_empty() || line.contains(HOST_KEY_QUESTION) || prompt_in(line).is_some()
}

impl Drop for SecretAnswer {
    fn drop(&mut self) {
        let mut bytes = std::mem::take(&mut self.secret).into_bytes();
        bytes.fill(0);
        black_box(&bytes);
    }
}
