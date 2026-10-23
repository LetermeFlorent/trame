use std::ops::Range;

use super::names::{host_names, is_filler, setting, starts_block};
use crate::preferences::SshHost;
use crate::preferences::ssh::DEFAULT_PORT;

pub(super) const HOST_NAME: &str = "hostname";
pub(super) const USER: &str = "user";
pub(super) const PORT: &str = "port";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostEntry {
    pub address: String,
    pub user: Option<String>,
    pub port: Option<u16>,
}

impl HostEntry {
    pub fn of(host: &SshHost) -> Self {
        let (user, address) = match host.target.split_once('@') {
            Some((user, address)) => (Some(user.to_owned()), address),
            None => (None, host.target.as_str()),
        };
        Self {
            address: address.to_owned(),
            user,
            port: host.port,
        }
    }
}

pub(super) fn lines(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

pub(super) fn block_of(lines: &[&str], name: &str) -> Option<Range<usize>> {
    let start = lines
        .iter()
        .position(|line| host_names(line).contains(&name))?;
    let body = start + 1;
    let next = lines[body..]
        .iter()
        .position(|line| starts_block(line))
        .map_or(lines.len(), |offset| body + offset);
    let mut end = next;
    while end > body && is_filler(lines[end - 1]) {
        end -= 1;
    }
    Some(start..end)
}

pub(super) fn is_setting(line: &str, keys: &[&str]) -> bool {
    setting(line).is_some_and(|(key, _)| keys.iter().any(|wanted| key.eq_ignore_ascii_case(wanted)))
}

pub fn entry_in(text: &str, name: &str) -> Option<HostEntry> {
    let lines = lines(text);
    let block = block_of(&lines, name)?;
    let (mut address, mut user, mut port) = (None, None, None);
    for (key, value) in lines[block.start + 1..block.end]
        .iter()
        .filter_map(|line| setting(line))
    {
        let key = key.to_ascii_lowercase();
        match key.as_str() {
            HOST_NAME => address = address.or(Some(value)),
            USER => user = user.or(Some(value)),
            PORT => port = port.or(Some(value)),
            _ => {}
        }
    }
    Some(HostEntry {
        address: address.unwrap_or(name).to_owned(),
        user: user.map(str::to_owned),
        port: port
            .and_then(|port| port.parse().ok())
            .filter(|&port| port != DEFAULT_PORT),
    })
}
