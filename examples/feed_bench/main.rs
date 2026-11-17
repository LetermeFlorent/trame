mod cases;
mod expect;
mod heap;
mod styled;

use std::time::{Duration, Instant};

use trame::term::{DEFAULT_HISTORY, Terminal};

#[global_allocator]
static HEAP: heap::CountingHeap = heap::CountingHeap;

const ROWS: usize = 40;
const COLS: usize = 120;
const SMALL_ROWS: usize = ROWS * 3 / 4;
const SMALL_COLS: usize = COLS * 3 / 4;
const FEED_CHUNK: usize = 64 * 1024;
const ROUNDS: usize = 10;
const RESIZE_CYCLES: u32 = 1_000;
const RESIZES_PER_CYCLE: u32 = 2;
const BYTES_PER_KIB: usize = 1024;
const BYTES_PER_MB: f64 = 1_000_000.0;
const MICROS_PER_SECOND: f64 = 1_000_000.0;

fn main() {
    println!(
        "feed bench: {ROWS}x{COLS} grid, {DEFAULT_HISTORY}-line history, {} KiB chunks, best of {ROUNDS} rounds",
        FEED_CHUNK / BYTES_PER_KIB
    );
    let cases = cases::all(ROWS, COLS, DEFAULT_HISTORY);
    let mut best_rates = vec![0.0_f64; cases.len()];
    let mut best_resize = f64::INFINITY;
    for round in 0..ROUNDS {
        for offset in 0..cases.len() {
            let index = (round + offset) % cases.len();
            let rate = megabytes(cases[index].input.len()) / feed_seconds(&cases[index]);
            best_rates[index] = best_rates[index].max(rate);
        }
        best_resize = best_resize.min(resize_micros());
    }
    for (case, rate) in cases.iter().zip(best_rates) {
        let size = megabytes(case.input.len());
        println!("{:<9} {size:>5.1} MB {rate:>8.1} MB/s", case.name);
    }
    println!(
        "{:<9} {best_resize:.3} us per resize between {ROWS}x{COLS} and {SMALL_ROWS}x{SMALL_COLS}, full history",
        "resize"
    );
    for (name, input) in [
        ("history", cases::plain_lines(ROWS + DEFAULT_HISTORY)),
        (
            "hist-var",
            cases::varied_lines(ROWS + DEFAULT_HISTORY, COLS),
        ),
    ] {
        let (live, compacted) = history_bytes(&input);
        println!(
            "{name:<9} {:.1} MB held by a full {DEFAULT_HISTORY}-line history, {:.1} MB after compact",
            megabytes(live),
            megabytes(compacted)
        );
    }
}

fn megabytes(bytes: usize) -> f64 {
    bytes as f64 / BYTES_PER_MB
}

fn feed(terminal: &mut Terminal, input: &[u8]) {
    for chunk in input.chunks(FEED_CHUNK) {
        terminal.feed(chunk);
    }
}

fn history_len(terminal: &Terminal) -> usize {
    terminal.screen().grid().history_len()
}

fn feed_seconds(case: &cases::Case) -> f64 {
    let mut terminal = Terminal::new(ROWS, COLS, DEFAULT_HISTORY);
    let start = Instant::now();
    feed(&mut terminal, &case.input);
    let seconds = start.elapsed().as_secs_f64();
    expect::check(&terminal, case);
    seconds
}

fn full_history(input: &[u8]) -> Terminal {
    let mut terminal = Terminal::new(ROWS, COLS, DEFAULT_HISTORY);
    feed(&mut terminal, input);
    assert_eq!(history_len(&terminal), DEFAULT_HISTORY);
    terminal
}

fn resize_micros() -> f64 {
    let mut terminal = full_history(&cases::plain_lines(ROWS + DEFAULT_HISTORY));
    let refill = cases::plain_lines(ROWS - SMALL_ROWS);
    let mut spent = Duration::ZERO;
    for _ in 0..RESIZE_CYCLES {
        let start = Instant::now();
        terminal.screen_mut().resize(SMALL_ROWS, SMALL_COLS);
        terminal.screen_mut().resize(ROWS, COLS);
        spent += start.elapsed();
        feed(&mut terminal, &refill);
    }
    assert_eq!(history_len(&terminal), DEFAULT_HISTORY);
    let resizing = spent.saturating_sub(clock_cost(RESIZE_CYCLES));
    resizing.as_secs_f64() * MICROS_PER_SECOND / f64::from(RESIZES_PER_CYCLE * RESIZE_CYCLES)
}

fn clock_cost(pairs: u32) -> Duration {
    (0..pairs).map(|_| Instant::now().elapsed()).sum()
}

fn history_bytes(input: &[u8]) -> (usize, usize) {
    let before = heap::live_bytes();
    let mut terminal = full_history(input);
    let live = heap::live_bytes() - before;
    terminal.compact();
    (live, heap::live_bytes() - before)
}
