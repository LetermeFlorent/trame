use std::ffi::{OsStr, OsString};
use std::iter::repeat_n;
use std::os::windows::ffi::OsStrExt;

use crate::session::pty::DUMP_VARIABLE;

const QUOTE: u16 = b'"' as u16;
const BACKSLASH: u16 = b'\\' as u16;
const SPACE: u16 = b' ' as u16;
const TAB: u16 = b'\t' as u16;
const EQUALS: u16 = b'=' as u16;

pub(super) fn command_line(program: &OsStr, args: &[OsString]) -> Vec<u16> {
    let mut line = Vec::new();
    append_argument(&mut line, program);
    for arg in args {
        line.push(SPACE);
        append_argument(&mut line, arg);
    }
    line.push(0);
    line
}

fn append_argument(line: &mut Vec<u16>, arg: &OsStr) {
    let units: Vec<u16> = arg.encode_wide().collect();
    let needs_quotes = units.is_empty()
        || units
            .iter()
            .any(|&unit| matches!(unit, SPACE | TAB | QUOTE));
    if !needs_quotes {
        line.extend_from_slice(&units);
        return;
    }
    line.push(QUOTE);
    let mut backslashes = 0;
    for unit in units {
        if unit == BACKSLASH {
            backslashes += 1;
            continue;
        }
        let escaped = if unit == QUOTE {
            backslashes * 2 + 1
        } else {
            backslashes
        };
        line.extend(repeat_n(BACKSLASH, escaped));
        line.push(unit);
        backslashes = 0;
    }
    line.extend(repeat_n(BACKSLASH, backslashes * 2));
    line.push(QUOTE);
}

pub(super) fn environment_block(extra: &[(OsString, OsString)]) -> Vec<u16> {
    let mut vars: Vec<(OsString, OsString)> = std::env::vars_os()
        .filter(|(key, _)| !extra.iter().any(|(name, _)| name.eq_ignore_ascii_case(key)))
        .collect();
    vars.extend(extra.iter().cloned());
    vars.retain(|(key, _)| !key.eq_ignore_ascii_case(DUMP_VARIABLE));
    vars.sort_by_cached_key(|(key, _)| key.to_ascii_uppercase());
    let mut block = Vec::new();
    for (key, value) in vars {
        block.extend(key.encode_wide());
        block.push(EQUALS);
        block.extend(value.encode_wide());
        block.push(0);
    }
    block.push(0);
    block
}
