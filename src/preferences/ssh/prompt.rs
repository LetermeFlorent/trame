use std::hint::black_box;
use std::time::{Duration, Instant};

pub const ANSWER_WINDOW: Duration = Duration::from_secs(60);

const PASSPHRASE_PROMPT: &str = "enter passphrase for key";
const PASSWORD_PROMPT: &str = "'s password:";
const INTERACTIVE_PROMPT: &str = ") password:";
const PROMPT_KINDS: usize = 2;

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
    deadline: Instant,
    answered: Vec<Prompt>,
}

impl SecretAnswer {
    pub fn new(secret: String, now: Instant) -> Self {
        Self {
            secret,
            deadline: now + ANSWER_WINDOW,
            answered: Vec::new(),
        }
    }

    pub fn reply(&mut self, line: &str, now: Instant) -> Option<String> {
        if self.is_over(now) {
            return None;
        }
        let prompt = prompt_in(line).filter(|prompt| !self.answered.contains(prompt))?;
        self.answered.push(prompt);
        Some(format!("{}\r", self.secret))
    }

    pub fn is_over(&self, now: Instant) -> bool {
        now >= self.deadline || self.answered.len() == PROMPT_KINDS
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }
}

impl Drop for SecretAnswer {
    fn drop(&mut self) {
        let mut bytes = std::mem::take(&mut self.secret).into_bytes();
        bytes.fill(0);
        black_box(&bytes);
    }
}
