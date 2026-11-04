mod builder;
mod curves;
mod reader;

use crate::geometry::Point;
use builder::Builder;
pub(super) use curves::ellipse;
use reader::Reader;

pub(super) fn flatten(d: &str) -> Vec<Vec<Point>> {
    let mut reader = Reader::new(d);
    let mut builder = Builder::default();
    let mut command = None;
    while !reader.done() {
        if let Some(letter) = reader.command() {
            command = Some(letter);
        } else if matches!(command, None | Some(b'Z' | b'z')) {
            break;
        }
        match command.and_then(|letter| builder.step(letter, &mut reader)) {
            Some(next) => command = Some(next),
            None => break,
        }
    }
    builder.finish();
    builder.lines
}
