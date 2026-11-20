use std::path::Path;

use super::Found;
use super::resolve::{overlaps, resolve_candidate, starts_chars};

const EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];
const SPACE_PREFIXES: [&str; 7] = ["file://", "~/", "~\\", "./", ".\\", "../", "..\\"];
const SKIP_DELIM: [&str; 4] = ["https://", "http://", "www.", "file://"];

pub(super) fn find_image_paths(
    text: &[char],
    claimed: &mut Vec<(usize, usize)>,
    cwd: Option<&Path>,
) -> Vec<Found> {
    find_image_paths_with(text, claimed, cwd, &mut Path::is_file)
}

pub(super) fn find_image_paths_with(
    text: &[char],
    claimed: &mut Vec<(usize, usize)>,
    cwd: Option<&Path>,
    is_file: &mut impl FnMut(&Path) -> bool,
) -> Vec<Found> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let Some((dot, ext_end)) = match_ext(text, i) else {
            i += 1;
            continue;
        };
        if !overlaps(claimed, dot, ext_end)
            && let Some(hit) = pick_candidate(text, (dot, ext_end), claimed, cwd, is_file)
        {
            claimed.push((hit.start, hit.end));
            found.push(hit);
        }
        i = ext_end;
    }
    found
}

fn match_ext(text: &[char], dot: usize) -> Option<(usize, usize)> {
    (text[dot] == '.').then_some(())?;
    EXTENSIONS.iter().find_map(|ext| {
        let end = dot + 1 + ext.len();
        (end <= text.len() && starts_chars(&text[dot + 1..end], ext) && is_path_end(text, end))
            .then_some((dot, end))
    })
}

fn is_path_end(text: &[char], end: usize) -> bool {
    let stop =
        |c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ')' | ']' | '}' | '>' | ',' | ';');
    match text.get(end) {
        None => true,
        Some('.') => text.get(end + 1).copied().is_none_or(stop),
        Some(&ch) => stop(ch) || matches!(ch, ':' | '!' | '?' | '`'),
    }
}

fn pick_candidate(
    text: &[char],
    (dot, end): (usize, usize),
    claimed: &[(usize, usize)],
    cwd: Option<&Path>,
    is_file: &mut dyn FnMut(&Path) -> bool,
) -> Option<Found> {
    let floor = claimed
        .iter()
        .fold(0, |max, &(_, e)| if e <= dot { max.max(e) } else { max });
    let token = token_start(text, floor, dot);
    let q = quoted_start(text, floor, dot, end);
    let a = anchor_start(text, floor, dot);
    let fixed = [q, a, token];
    let mut seen = Vec::new();
    let delims = token.into_iter().flat_map(|s| after_delim(text, s, dot));
    let mut iter = fixed.into_iter().flatten().chain(delims);
    iter.find_map(|start| {
        (!seen.contains(&start) && !overlaps(claimed, start, end)).then_some(())?;
        seen.push(start);
        let raw: String = text[start..end].iter().collect();
        let path = resolve_candidate(&raw, cwd, &mut *is_file)?;
        let target = path.to_string_lossy().into_owned();
        Some(Found { start, end, target })
    })
}

fn quoted_start(text: &[char], floor: usize, dot: usize, end: usize) -> Option<usize> {
    let quote = *text.get(end).filter(|c| matches!(c, '"' | '\'' | '`'))?;
    let open = (floor..dot).rfind(|&i| text[i] == quote || text[i].is_control())?;
    (text[open] == quote && open + 1 < dot).then_some(open + 1)
}

fn starts_any(text: &[char], prefixes: &[&str]) -> bool {
    prefixes.iter().any(|p| starts_chars(text, p))
}

fn is_drive(text: &[char], a: usize, dot: usize) -> bool {
    a + 2 < dot
        && text[a].is_ascii_alphabetic()
        && text[a + 1] == ':'
        && matches!(text[a + 2], '\\' | '/')
}

fn anchor_start(text: &[char], floor: usize, dot: usize) -> Option<usize> {
    (floor..dot)
        .rev()
        .take_while(|&a| !text[a].is_control())
        .find(|&a| {
            let colon = (is_drive(text, a, dot) || starts_any(&text[a..dot], &SPACE_PREFIXES))
                && delim(text, a, true);
            colon || (text[a] == '/' && delim(text, a, false))
        })
}

fn is_wrap_delim(ch: char) -> bool {
    matches!(ch, '(' | '[' | '{' | '<' | '=' | ',' | ';')
}

fn token_start(text: &[char], floor: usize, dot: usize) -> Option<usize> {
    let stop = |c: char| c.is_whitespace() || c.is_control() || matches!(c, '"' | '\'' | '`');
    let prev = (floor..dot).rfind(|&i| stop(text[i]));
    let mut start = prev.map_or(floor, |i| i + 1);
    while start < dot && is_wrap_delim(text[start]) {
        start += 1;
    }
    (start < dot).then_some(start)
}

fn after_delim(text: &[char], start: usize, dot: usize) -> impl Iterator<Item = usize> + '_ {
    let skip = starts_any(&text[start..dot], &SKIP_DELIM);
    let end = if skip { start } else { dot };
    (start..end).rev().filter_map(move |pos| {
        let colon = text[pos] == ':';
        let sep = colon || is_wrap_delim(text[pos]);
        let scheme = colon && starts_chars(&text[pos + 1..dot], "//");
        let drive = colon && pos > start && is_drive(text, pos - 1, dot);
        (sep && !scheme && !drive && pos + 1 < dot).then_some(pos + 1)
    })
}

fn delim(text: &[char], idx: usize, allow_colon: bool) -> bool {
    text[..idx].last().is_none_or(|&ch| {
        ch.is_whitespace()
            || is_wrap_delim(ch)
            || matches!(ch, '"' | '\'' | '`')
            || (allow_colon && matches!(ch, ':' | '>'))
    })
}
