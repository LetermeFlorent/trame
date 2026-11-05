mod config;
mod edit;
mod entry;
mod forget;
mod form;
mod list;
mod prompt;
mod vault;

use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use trame::preferences::SshHost;

pub const MANY: usize = 20;

pub struct TempFolder(PathBuf);

impl TempFolder {
    pub fn new(name: &str) -> Self {
        Self(env::temp_dir().join(format!("trame-ssh-{name}-{}", process::id())))
    }
}

impl Deref for TempFolder {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TempFolder {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn host(target: &str, port: Option<u16>) -> SshHost {
    SshHost {
        target: target.to_owned(),
        port,
    }
}

pub fn hosts(count: usize) -> Vec<SshHost> {
    (0..count)
        .map(|index| host(&format!("user@host{index}"), None))
        .collect()
}
