use std::ops::Range;

use crate::preferences::SshHost;

const HOST_KEYWORD: &str = "host";
const MATCH_KEYWORD: &str = "match";
const NOT_A_NAME: [char; 5] = ['*', '?', '!', '@', ':'];
const COMMENT: char = '#';
pub(super) const BYTE_ORDER_MARK: char = '\u{feff}';
const QUOTE: char = '"';

fn split_keyword(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_start_matches(BYTE_ORDER_MARK).trim();
    if line.is_empty() || line.starts_with(COMMENT) {
        return None;
    }
    let split = line
        .find(|ch: char| ch.is_whitespace() || ch == '=')
        .unwrap_or(line.len());
    let (keyword, rest) = line.split_at(split);
    Some((
        keyword,
        rest.trim_start().trim_start_matches('=').trim_start(),
    ))
}

pub(super) fn setting(line: &str) -> Option<(&str, &str)> {
    let (keyword, rest) = split_keyword(line)?;
    let value = rest.split_whitespace().next().unwrap_or_default();
    Some((keyword, value.trim_matches(QUOTE)))
}

pub(super) fn starts_block(line: &str) -> bool {
    split_keyword(line).is_some_and(|(keyword, _)| {
        keyword.eq_ignore_ascii_case(HOST_KEYWORD) || keyword.eq_ignore_ascii_case(MATCH_KEYWORD)
    })
}

pub(super) fn is_filler(line: &str) -> bool {
    split_keyword(line).is_none()
}

pub(super) fn host_names(line: &str) -> Vec<&str> {
    let Some((keyword, rest)) = split_keyword(line) else {
        return Vec::new();
    };
    if !keyword.eq_ignore_ascii_case(HOST_KEYWORD) {
        return Vec::new();
    }
    rest.split_whitespace()
        .take_while(|name| !name.starts_with(COMMENT))
        .map(|name| name.trim_matches(QUOTE))
        .filter(|name| !name.contains(NOT_A_NAME) && SshHost::parse(name).is_some())
        .collect()
}

pub(super) fn name_range(line: &str, name: &str) -> Option<Range<usize>> {
    let found = host_names(line).into_iter().find(|found| *found == name)?;
    let start = found.as_ptr() as usize - line.as_ptr() as usize;
    Some(start..start + found.len())
}

pub(super) fn quoted_range(line: &str, range: Range<usize>) -> Range<usize> {
    let opens = line[..range.start].ends_with(QUOTE);
    let closes = line[range.end..].starts_with(QUOTE);
    match opens && closes {
        true => range.start - 1..range.end + 1,
        false => range,
    }
}

pub fn aliases_in(text: &str) -> Vec<String> {
    let mut aliases: Vec<String> = Vec::new();
    for name in text.lines().flat_map(host_names) {
        if !aliases.iter().any(|alias| alias == name) {
            aliases.push(name.to_owned());
        }
    }
    aliases
}

pub fn menu_hosts(saved: &[SshHost], aliases: &[String]) -> Vec<SshHost> {
    let mut hosts = saved.to_vec();
    for host in aliases.iter().filter_map(|alias| SshHost::parse(alias)) {
        if !hosts.contains(&host) {
            hosts.push(host);
        }
    }
    hosts
}
