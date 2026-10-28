mod config;
mod form;
mod prompt;
pub mod vault;

use serde::{Deserialize, Serialize};

use super::Preferences;
pub use config::{
    HostEntry, SshConfig, aliases_in, appended, entry_in, menu_hosts, with_host, without_host,
    written_by_hand,
};
pub use form::{ConnectForm, FormField};
pub use prompt::{ANSWER_WINDOW, Prompt, SecretAnswer, prompt_in};

const DEFAULT_PORT: u16 = 22;

fn custom_port(port: Option<u16>) -> Option<u16> {
    port.filter(|&port| port != DEFAULT_PORT)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshHost {
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

impl SshHost {
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let (target, port) = match text.rsplit_once(':') {
            Some((target, port)) => (target, Some(port.parse().ok()?)),
            None => (text, None),
        };
        let host = Self {
            target: target.to_owned(),
            port: custom_port(port),
        };
        host.is_valid().then_some(host)
    }

    pub fn from_parts(address: &str, user: &str, port: &str) -> Option<Self> {
        let parsed = Self::parse(address)?;
        let target = match user.trim() {
            "" => parsed.target,
            _ if parsed.target.contains('@') => return None,
            user => format!("{user}@{}", parsed.target),
        };
        let port = match port.trim() {
            "" => parsed.port,
            port => Some(port.parse().ok()?),
        };
        let host = Self {
            target,
            port: custom_port(port),
        };
        host.is_valid().then_some(host)
    }

    fn split(&self) -> (Option<&str>, &str) {
        match self.target.split_once('@') {
            Some((user, name)) => (Some(user), name),
            None => (None, self.target.as_str()),
        }
    }

    pub fn label(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{port}", self.target),
            None => self.target.clone(),
        }
    }

    pub fn alias(&self) -> Option<&str> {
        let bare = self.port.is_none() && self.split().0.is_none();
        bare.then_some(self.target.as_str())
    }

    pub fn command(&self) -> String {
        match self.port {
            Some(port) => format!("ssh -p {port} {}\r", self.target),
            None => format!("ssh {}\r", self.target),
        }
    }

    pub fn is_valid(&self) -> bool {
        let mut parts = self.target.split('@');
        let (Some(first), second, None) = (parts.next(), parts.next(), parts.next()) else {
            return false;
        };
        let named = |part: &str| {
            !part.is_empty()
                && !part.starts_with('-')
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        };
        named(first) && second.is_none_or(named) && self.port != Some(0)
    }
}

impl Preferences {
    pub fn remember_host(&self, host: SshHost) -> Self {
        let mut ssh_hosts: Vec<SshHost> = self
            .ssh_hosts
            .iter()
            .filter(|known| **known != host)
            .cloned()
            .collect();
        ssh_hosts.insert(0, host);
        self.with(|next| next.ssh_hosts = ssh_hosts)
    }

    pub fn replace_host(&self, old: &SshHost, host: SshHost) -> Self {
        let Some(at) = self.ssh_hosts.iter().position(|known| known == old) else {
            return self.remember_host(host);
        };
        let ssh_hosts = self
            .ssh_hosts
            .iter()
            .enumerate()
            .filter(|&(index, known)| index == at || *known != host)
            .map(|(index, known)| match index == at {
                true => host.clone(),
                false => known.clone(),
            })
            .collect();
        self.with(|next| next.ssh_hosts = ssh_hosts)
    }

    pub fn forget_host(&self, host: &SshHost) -> Self {
        self.with(|next| next.ssh_hosts.retain(|known| known != host))
    }
}

pub(super) fn sanitized_hosts(hosts: Vec<SshHost>) -> Vec<SshHost> {
    let mut kept: Vec<SshHost> = Vec::new();
    for host in hosts {
        let host = SshHost {
            port: custom_port(host.port),
            ..host
        };
        if host.is_valid() && !kept.contains(&host) {
            kept.push(host);
        }
    }
    kept
}
