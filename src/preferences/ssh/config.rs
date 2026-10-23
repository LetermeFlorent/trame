use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use super::SshHost;

const HOST_KEYWORD: &str = "host";
const NOT_A_NAME: [char; 5] = ['*', '?', '!', '@', ':'];
const COMMENT: char = '#';
const BYTE_ORDER_MARK: char = '\u{feff}';
const INDENT: &str = "    ";

pub struct SshConfig {
    path: PathBuf,
}

impl SshConfig {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn aliases(&self) -> io::Result<Vec<String>> {
        Ok(aliases_in(&self.text()?))
    }

    pub fn keep(&self, host: &SshHost) -> io::Result<SshHost> {
        let existing = self.text()?;
        let name = name_of(host);
        if aliases_in(&existing).iter().any(|alias| alias == name) {
            return Ok(host.clone());
        }
        if let Some(folder) = self.path.parent() {
            fs::create_dir_all(folder)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(appended(&existing, host).as_bytes())?;
        Ok(SshHost {
            target: name.to_owned(),
            port: None,
        })
    }

    fn text(&self) -> io::Result<String> {
        match fs::read_to_string(&self.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
            read => read,
        }
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

fn host_names(line: &str) -> Vec<&str> {
    let line = line.trim_start_matches(BYTE_ORDER_MARK).trim();
    let split = line
        .find(|ch: char| ch.is_whitespace() || ch == '=')
        .unwrap_or(line.len());
    let (keyword, rest) = line.split_at(split);
    if !keyword.eq_ignore_ascii_case(HOST_KEYWORD) {
        return Vec::new();
    }
    rest.trim_start()
        .trim_start_matches('=')
        .split_whitespace()
        .take_while(|name| !name.starts_with(COMMENT))
        .map(|name| name.trim_matches('"'))
        .filter(|name| !name.contains(NOT_A_NAME) && SshHost::parse(name).is_some())
        .collect()
}

fn name_of(host: &SshHost) -> &str {
    host.target
        .split_once('@')
        .map_or(host.target.as_str(), |(_, name)| name)
}

pub fn appended(existing: &str, host: &SshHost) -> String {
    let newline = match existing.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    };
    let lead = match existing {
        text if text.trim().is_empty() => String::new(),
        text if text.ends_with('\n') => newline.to_owned(),
        _ => newline.repeat(2),
    };
    let name = name_of(host);
    let user = host.target.split_once('@').map(|(user, _)| user);
    let mut lines = vec![format!("Host {name}"), format!("{INDENT}HostName {name}")];
    lines.extend(user.map(|user| format!("{INDENT}User {user}")));
    lines.extend(host.port.map(|port| format!("{INDENT}Port {port}")));
    format!("{lead}{}{newline}", lines.join(newline))
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
