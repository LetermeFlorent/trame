use std::sync::Mutex;

use super::MAX_TRANSCRIPT_BYTES;
use super::encode::{encode, least_bytes};
use crate::session::lock;
use crate::term::Terminal;
use crate::term::grid::{Grid, Row};

pub struct TranscriptCopy {
    generation: u64,
    cols: usize,
    rows: Vec<Row>,
}

impl TranscriptCopy {
    pub fn take(terminal: &Mutex<Terminal>) -> Self {
        Self::of(&lock(terminal))
    }

    pub fn of(terminal: &Terminal) -> Self {
        let grid = &terminal.screen().primary;
        Self {
            generation: terminal.generation(),
            cols: grid.cols(),
            rows: tail_rows(grid, MAX_TRANSCRIPT_BYTES),
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn encode(&self, budget: usize) -> Vec<u8> {
        encode(&self.rows, self.cols, budget)
    }
}

fn tail_rows(grid: &Grid, budget: usize) -> Vec<Row> {
    let Some(last) = last_written_line(grid) else {
        return Vec::new();
    };
    let mut used = 0;
    let mut rows: Vec<Row> = (grid.first_line()..=last)
        .rev()
        .filter_map(|line| grid.line(line))
        .enumerate()
        .take_while(|(index, row)| {
            used += least_bytes(row, grid.cols(), *index == 0);
            used <= budget
        })
        .map(|(_, row)| row.clone())
        .collect();
    rows.reverse();
    rows
}

fn last_written_line(grid: &Grid) -> Option<u64> {
    (grid.first_line()..grid.end_line())
        .rev()
        .find(|line| grid.line(*line).is_some_and(|row| !row.is_blank()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_tail_that_fits_the_budget_is_copied() {
        let mut terminal = Terminal::new(4, 30, 1000);
        for line in 0..500 {
            terminal.feed(format!("\x1b[1mline {line}\x1b[0m\r\n").as_bytes());
        }
        terminal.feed("\u{4E2D}\u{6587}".repeat(20).as_bytes());
        let grid = &terminal.screen().primary;
        let every = tail_rows(grid, usize::MAX);
        for budget in [0, 1, 40, 41, 100, 1000, 3000] {
            let tail = tail_rows(grid, budget);
            assert!(tail.len() < every.len());
            assert_eq!(encode(&tail, 30, budget), encode(&every, 30, budget));
        }
    }
}
