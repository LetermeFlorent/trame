use serde::{Deserialize, Serialize};

use super::Preferences;

pub const MAX_SSH_HOSTS: usize = 8;

const DEFAULT_PORT: u16 = 22;

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
            port: port.filter(|&port| port != DEFAULT_PORT),
        };
        host.is_valid().then_some(host)
    }

    pub fn label(&self) -> String {
        match self.port {
            Some(port) => format!("{}:{port}", self.target),
            None => self.target.clone(),
        }
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
        ssh_hosts.truncate(MAX_SSH_HOSTS);
        Self {
            ssh_hosts,
            ..self.clone()
        }
    }

    pub fn forget_host(&self, index: usize) -> Self {
        let mut ssh_hosts = self.ssh_hosts.clone();
        if index < ssh_hosts.len() {
            ssh_hosts.remove(index);
        }
        Self {
            ssh_hosts,
            ..self.clone()
        }
    }
}

pub(super) fn sanitized_hosts(hosts: Vec<SshHost>) -> Vec<SshHost> {
    let mut kept: Vec<SshHost> = Vec::new();
    for host in hosts {
        let host = SshHost {
            port: host.port.filter(|&port| port != DEFAULT_PORT),
            ..host
        };
        if host.is_valid() && !kept.contains(&host) && kept.len() < MAX_SSH_HOSTS {
            kept.push(host);
        }
    }
    kept
}
