use trame::term::Screen;
use trame::term::SelectionKind::{self, Line, Simple, Word};
use trame::term::vt::{Params, Parser, Perform};

use super::random_input::{Rng, random_input};

const CASES: u64 = 2_000;
const MAX_ROWS: usize = 5;
const MAX_COLS: usize = 12;
const MAX_HISTORY: usize = 3;
const MAX_CHUNK: usize = 16;
const MIN_FAST_PATH_PERCENT: usize = 40;
const SELECTION_KINDS: [SelectionKind; 3] = [Simple, Word, Line];

#[derive(Default)]
struct RunBytes {
    fast_path: usize,
    all: usize,
}

impl RunBytes {
    fn count(&mut self, len: usize, fast_path: bool) {
        self.all += len;
        self.fast_path += if fast_path { len } else { 0 };
    }

    fn assert_share(&self, printed: &str) {
        let RunBytes { fast_path, all } = self;
        let percent = fast_path * 100 / all;
        println!("fast path printed {fast_path} of {all} run bytes {printed} ({percent}%)");
        assert!(percent >= MIN_FAST_PATH_PERCENT, "{printed}: {percent}%");
    }
}

#[derive(Default)]
struct Shares {
    every_state: RunBytes,
    attributes_set: RunBytes,
}

struct PerChar<'a> {
    screen: &'a mut Screen,
    runs: Vec<Vec<u8>>,
    shares: &'a mut Shares,
}

impl Perform for PerChar<'_> {
    fn print(&mut self, c: char) {
        self.screen.print(c);
    }

    fn print_ascii(&mut self, text: &[u8]) {
        self.runs.push(text.to_vec());
        let (len, fast_path) = (text.len(), self.screen.prints_ascii_as_is());
        self.shares.every_state.count(len, fast_path);
        if self.screen.template_sets_attributes() {
            self.shares.attributes_set.count(len, fast_path);
        }
        text.iter().for_each(|&byte| self.screen.print(byte.into()));
    }

    fn execute(&mut self, byte: u8) {
        self.screen.execute(byte);
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], action: u8) {
        self.screen.csi_dispatch(params, intermediates, action);
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], byte: u8) {
        self.screen.esc_dispatch(intermediates, byte);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        self.screen.osc_dispatch(params, bell_terminated);
    }
}

fn per_char<'a>(screen: &'a mut Screen, shares: &'a mut Shares) -> PerChar<'a> {
    PerChar {
        screen,
        runs: Vec::new(),
        shares,
    }
}

fn feed_in_chunks(rng: &mut Rng, parser: &mut Parser, screen: &mut Screen, input: &[u8]) {
    let mut rest = input;
    while !rest.is_empty() {
        let (chunk, tail) = rest.split_at(rest.len().min(rng.below(MAX_CHUNK) + 1));
        parser.advance(screen, chunk);
        rest = tail;
    }
}

fn select_at_random(rng: &mut Rng, screens: [&mut Screen; 2], rows: usize, cols: usize) {
    let kind = SELECTION_KINDS[rng.below(SELECTION_KINDS.len())];
    let start = (rng.below(rows), rng.below(cols));
    let end = (rng.below(rows), rng.below(cols));
    for screen in screens {
        screen.start_selection(kind, start.0, start.1);
        screen.extend_selection(end.0, end.1);
    }
}

#[test]
fn ascii_runs_print_like_one_char_at_a_time() {
    let mut shares = Shares::default();
    for seed in 1..=CASES {
        let mut rng = Rng::new(seed);
        let (rows, cols) = (rng.below(MAX_ROWS) + 1, rng.below(MAX_COLS) + 1);
        let history = rng.below(MAX_HISTORY + 1);
        let input = random_input(&mut rng, rows, cols);
        let (before, after) = input.split_at(rng.below(input.len() + 1));
        let mut fast = Screen::new(rows, cols, history);
        let mut slow_screen = fast.clone();
        let mut slow = per_char(&mut slow_screen, &mut shares);
        let (mut fast_parser, mut slow_parser) = (Parser::new(), Parser::new());
        feed_in_chunks(&mut rng, &mut fast_parser, &mut fast, before);
        slow_parser.advance(&mut slow, before);
        select_at_random(&mut rng, [&mut fast, &mut *slow.screen], rows, cols);
        feed_in_chunks(&mut rng, &mut fast_parser, &mut fast, after);
        slow_parser.advance(&mut slow, after);
        let shown = String::from_utf8_lossy(&input);
        let same = format!("{fast:?}") == format!("{:?}", slow.screen);
        assert!(same, "seed {seed}, {rows}x{cols}: {shown:?}");
    }
    shares.every_state.assert_share("in any state");
    shares
        .attributes_set
        .assert_share("while attributes are set");
}

#[test]
fn printable_ascii_reaches_the_performer_in_runs() {
    let mut screen = Screen::new(2, 20, 0);
    let mut shares = Shares::default();
    let mut recorded = per_char(&mut screen, &mut shares);
    let mut parser = Parser::new();
    parser.advance(&mut recorded, b"ab\x1b[1mcd\x7fe\xC3fg");
    parser.advance(&mut recorded, b"h");
    let runs: Vec<&[u8]> = recorded.runs.iter().map(Vec::as_slice).collect();
    assert_eq!(runs, [&b"ab"[..], b"cd", b"e", b"g", b"h"]);
    assert_eq!(screen.grid().row(0).text(), "abcde\u{FFFD}fgh");
}
