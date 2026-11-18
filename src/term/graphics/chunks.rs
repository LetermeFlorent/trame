use super::{Command, Error};
use crate::term::dispatch::decode::decode_base64;

pub const MAX_ENCODED: usize = 64 << 20;

#[derive(Debug, Clone)]
pub struct Chunks {
    first: Command,
    data: Vec<u8>,
}

impl Chunks {
    pub fn start(first: Command) -> Self {
        Self {
            first,
            data: Vec::new(),
        }
    }

    pub fn first(&self) -> &Command {
        &self.first
    }

    pub fn set_quiet(&mut self, quiet: u8) {
        self.first.quiet = quiet;
    }

    pub fn push(&mut self, more: bool, b64: &[u8]) -> Result<Option<(Command, Vec<u8>)>, Error> {
        if self.data.len() + b64.len() > MAX_ENCODED {
            self.data = Vec::new();
            return Err(Error::TooBig);
        }
        self.data.extend_from_slice(b64);
        if more {
            return Ok(None);
        }
        let raw = decode_base64(&std::mem::take(&mut self.data)).ok_or(Error::Inval)?;
        let mut command = self.first.clone();
        command.more = false;
        Ok(Some((command, raw)))
    }
}
