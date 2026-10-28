use super::{INTERACTIVE_PROMPT, PASSWORD_PROMPT};
use crate::preferences::{HostEntry, SshHost};

pub struct SshLogin {
    user: Option<String>,
    hosts: Vec<String>,
    command: String,
}

impl SshLogin {
    pub fn of(host: &SshHost, entry: Option<&HostEntry>) -> Self {
        let (user, name) = host.split();
        let user = user.or_else(|| entry.and_then(|entry| entry.user.as_deref()));
        let mut hosts = vec![name.to_lowercase()];
        hosts.extend(entry.map(|entry| entry.address.to_lowercase()));
        Self {
            user: user.map(str::to_lowercase),
            hosts,
            command: host.command().trim_end().to_lowercase(),
        }
    }

    pub(super) fn is_asked_by(&self, line: &str) -> bool {
        login_in(line).is_some_and(|(user, host)| {
            self.hosts.iter().any(|known| known == host)
                && self.user.as_deref().is_none_or(|known| known == user)
        })
    }

    pub(super) fn is_typed_in(&self, line: &str) -> bool {
        line.contains(&self.command)
    }
}

fn login_in(line: &str) -> Option<(&str, &str)> {
    let interactive = || line.strip_prefix('(')?.strip_suffix(INTERACTIVE_PROMPT);
    line.strip_suffix(PASSWORD_PROMPT)
        .or_else(interactive)?
        .rsplit_once('@')
}
