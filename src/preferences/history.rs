use super::Preferences;

pub const HISTORY_CHOICES: [usize; 6] = [1_000, 2_000, 5_000, 10_000, 20_000, 50_000];
pub const DEFAULT_HISTORY_LINES: usize = HISTORY_CHOICES[0];

impl Preferences {
    pub fn step_history_lines(&self, steps: i32) -> Self {
        let last = HISTORY_CHOICES.len() as i32 - 1;
        let index = (choice_index(self.history_lines) as i32 + steps).clamp(0, last);
        self.with(|next| next.history_lines = HISTORY_CHOICES[index as usize])
    }
}

pub(super) fn sanitized_history(lines: usize) -> usize {
    HISTORY_CHOICES[choice_index(lines)]
}

pub(super) fn history_lines() -> usize {
    DEFAULT_HISTORY_LINES
}

fn choice_index(lines: usize) -> usize {
    HISTORY_CHOICES
        .iter()
        .rposition(|&choice| choice <= lines)
        .unwrap_or(0)
}
