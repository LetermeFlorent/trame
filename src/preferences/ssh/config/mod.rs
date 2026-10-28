mod block;
mod edit;
mod names;

use std::fs;
use std::io;
use std::path::PathBuf;

use super::SshHost;
use crate::files::write_atomic;
pub use block::{HostEntry, entry_in};
pub use edit::{appended, with_host, without_host, written_by_hand};
pub use names::{aliases_in, menu_hosts};

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

    pub fn entry(&self, name: &str) -> io::Result<Option<HostEntry>> {
        Ok(entry_in(&self.text()?, name))
    }

    pub fn keep(&self, host: &SshHost) -> io::Result<SshHost> {
        let existing = self.text()?;
        let name = host.split().1;
        if aliases_in(&existing).iter().any(|alias| alias == name) {
            return Ok(host.clone());
        }
        let next = format!("{existing}{}", appended(&existing, host));
        write_atomic(&self.path, next.as_bytes())?;
        Ok(SshHost {
            target: name.to_owned(),
            port: None,
        })
    }

    pub fn written_by_hand(&self, name: &str) -> io::Result<bool> {
        Ok(written_by_hand(&self.text()?, name))
    }

    pub fn remove(&self, name: &str) -> io::Result<()> {
        let text = self.text()?;
        let next = without_host(&text, name);
        match next == text {
            true => Ok(()),
            false => write_atomic(&self.path, next.as_bytes()),
        }
    }

    pub fn replace(&self, name: &str, host: &SshHost) -> io::Result<SshHost> {
        let text = self.text()?;
        let Some((next, alias)) = with_host(&text, name, host) else {
            return self.keep(host);
        };
        if next != text {
            write_atomic(&self.path, next.as_bytes())?;
        }
        Ok(SshHost {
            target: alias,
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
