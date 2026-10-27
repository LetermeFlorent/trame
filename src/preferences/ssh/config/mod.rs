mod block;
mod edit;
mod names;

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use super::SshHost;
pub use block::{HostEntry, entry_in};
pub use edit::{appended, with_host, without_host};
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

    pub fn remove(&self, name: &str) -> io::Result<()> {
        let text = self.text()?;
        let next = without_host(&text, name);
        match next == text {
            true => Ok(()),
            false => fs::write(&self.path, next),
        }
    }

    pub fn replace(&self, name: &str, host: &SshHost) -> io::Result<SshHost> {
        let text = self.text()?;
        let Some((next, alias)) = with_host(&text, name, host) else {
            return self.keep(host);
        };
        if next != text {
            fs::write(&self.path, next)?;
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
