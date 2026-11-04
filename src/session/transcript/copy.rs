use std::sync::Mutex;

use super::encode::encode;
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
        let rows = last_written_line(grid).map_or_else(Vec::new, |last| {
            (grid.first_line()..=last)
                .filter_map(|line| grid.line(line).cloned())
                .collect()
        });
        Self {
            generation: terminal.generation(),
            cols: grid.cols(),
            rows,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn encode(&self, budget: usize) -> Vec<u8> {
        encode(&self.rows, self.cols, budget)
    }
}

fn last_written_line(grid: &Grid) -> Option<u64> {
    (grid.first_line()..grid.end_line())
        .rev()
        .find(|line| grid.line(*line).is_some_and(|row| !row.is_blank()))
}
