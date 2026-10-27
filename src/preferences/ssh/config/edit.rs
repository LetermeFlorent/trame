use std::borrow::Cow;

use super::block::{HOST_NAME, HostEntry, PORT, USER, block_of, entry_in, is_setting, lines};
use super::names::{BYTE_ORDER_MARK, aliases_in, host_names, is_filler, name_range, quoted_range};
use crate::preferences::SshHost;

const INDENT: &str = "    ";

fn newline_of(text: &str) -> &'static str {
    match text.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    }
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn settings_of(host: &SshHost, indent: &str, newline: &str) -> Vec<String> {
    let (user, name) = host.split();
    let mut lines = vec![format!("{indent}HostName {name}{newline}")];
    lines.extend(user.map(|user| format!("{indent}User {user}{newline}")));
    lines.extend(
        host.port
            .map(|port| format!("{indent}Port {port}{newline}")),
    );
    lines
}

pub fn appended(existing: &str, host: &SshHost) -> String {
    let newline = newline_of(existing);
    let lead = match existing {
        text if text.trim().is_empty() => String::new(),
        text if text.ends_with('\n') => newline.to_owned(),
        _ => newline.repeat(2),
    };
    let head = format!("Host {}{newline}", host.split().1);
    let body = settings_of(host, INDENT, newline).concat();
    format!("{lead}{head}{body}")
}

pub fn without_host(text: &str, name: &str) -> String {
    let body = text.strip_prefix(BYTE_ORDER_MARK).unwrap_or(text);
    let mark = &text[..text.len() - body.len()];
    let mut body = body.to_owned();
    while let Some(next) = removed_once(&body, name) {
        body = next;
    }
    format!("{mark}{body}")
}

fn removed_once(text: &str, name: &str) -> Option<String> {
    let lines = lines(text);
    let block = block_of(&lines, name)?;
    let head = lines[block.start];
    let mut kept: Vec<Cow<str>> = lines.iter().map(|line| Cow::Borrowed(*line)).collect();
    if host_names(head).len() > 1 {
        let range = quoted_range(head, name_range(head, name)?);
        let cut = head[..range.start].trim_end().len();
        kept[block.start] = Cow::Owned(format!("{}{}", &head[..cut], &head[range.end..]));
        return Some(kept.concat());
    }
    kept.drain(block.clone());
    let at = block.start;
    while at < kept.len() && is_blank(&kept[at]) && (at == 0 || is_blank(&kept[at - 1])) {
        kept.remove(at);
    }
    if at == kept.len() {
        while kept.last().is_some_and(|line| is_blank(line)) {
            kept.pop();
        }
    }
    Some(kept.concat())
}

fn alias_for<'a>(text: &str, name: &'a str, address: &'a str, old: &HostEntry) -> &'a str {
    let taken = aliases_in(text)
        .iter()
        .any(|alias| alias == address && alias != name);
    match old.address == name && !taken {
        true => address,
        false => name,
    }
}

fn indent_of<'a>(body: &[&'a str]) -> &'a str {
    body.iter()
        .copied()
        .find(|line| !is_filler(line))
        .map_or(INDENT, |line| &line[..line.len() - line.trim_start().len()])
}

fn renamed_head(head: &str, name: &str, alias: &str, newline: &str) -> Option<String> {
    let mut head = head.to_owned();
    if alias != name {
        let range = name_range(&head, name)?;
        head.replace_range(range, alias);
    }
    if !head.ends_with('\n') {
        head.push_str(newline);
    }
    Some(head)
}

pub fn with_host(text: &str, name: &str, host: &SshHost) -> Option<(String, String)> {
    let lines = lines(text);
    let block = block_of(&lines, name)?;
    let old = entry_in(text, name)?;
    let alias = alias_for(text, name, host.split().1, &old);
    let newline = newline_of(text);
    let body = &lines[block.start + 1..block.end];
    let head = renamed_head(lines[block.start], name, alias, newline)?;
    let kept = body
        .iter()
        .filter(|line| !is_setting(line, &[HOST_NAME, USER, PORT]))
        .map(|line| (*line).to_owned());
    let block_text: String = std::iter::once(head)
        .chain(settings_of(host, indent_of(body), newline))
        .chain(kept)
        .collect();
    let next = [
        lines[..block.start].concat(),
        block_text,
        lines[block.end..].concat(),
    ]
    .concat();
    Some((next, alias.to_owned()))
}
