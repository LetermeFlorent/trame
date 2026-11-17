use std::io::{BufRead, BufReader, Read};

const REDRAW: u8 = b'\r';
const PERCENT: char = '%';
const FULL: f32 = 100.0;

pub fn follow(stream: impl Read, on_percent: &dyn Fn(u8)) {
    let mut last = None;
    for update in BufReader::new(stream).split(REDRAW).map_while(Result::ok) {
        let Some(percent) = percent(&update).filter(|&percent| last != Some(percent)) else {
            continue;
        };
        last = Some(percent);
        on_percent(percent);
    }
}

fn percent(update: &[u8]) -> Option<u8> {
    let text = String::from_utf8_lossy(update);
    let number = text.split_whitespace().last()?.strip_suffix(PERCENT)?;
    let value: f32 = number.parse().ok()?;
    Some(value.clamp(0.0, FULL) as u8)
}
