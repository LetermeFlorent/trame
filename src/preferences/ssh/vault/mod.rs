#[cfg(windows)]
mod windows;

use std::io;

use super::SshHost;

const SSH_PREFIX: &str = "trame/ssh/";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vault {
    prefix: &'static str,
}

pub const SSH: Vault = Vault { prefix: SSH_PREFIX };

impl Vault {
    pub const fn with_prefix(prefix: &'static str) -> Self {
        Self { prefix }
    }

    pub fn key(self, host: &SshHost) -> String {
        format!("{}{}", self.prefix, host.label())
    }

    pub fn store(self, host: &SshHost, secret: &str) -> io::Result<()> {
        backend::store(&self.key(host), &host.target, secret)
    }

    pub fn load(self, host: &SshHost) -> Option<String> {
        backend::load(&self.key(host))
    }

    pub fn forget(self, host: &SshHost) {
        backend::forget(&self.key(host));
    }
}

#[cfg(windows)]
use windows as backend;

#[cfg(not(windows))]
mod backend {
    use std::io;

    pub(super) fn store(_key: &str, _user: &str, _secret: &str) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no credential store on this system",
        ))
    }

    pub(super) fn load(_key: &str) -> Option<String> {
        None
    }

    pub(super) fn forget(_key: &str) {}
}
