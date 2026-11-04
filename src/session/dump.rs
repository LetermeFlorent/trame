use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::SessionId;
use super::pty::DUMP_VARIABLE;
use crate::files::{create_file, create_folder};

const MAX_FILE_BYTES: usize = 64 * 1024 * 1024;

pub struct Dump {
    file: Option<File>,
    room: usize,
}

impl Dump {
    pub fn open(id: SessionId) -> Self {
        Self {
            file: Self::create(id),
            room: MAX_FILE_BYTES,
        }
    }

    pub fn record(&mut self, bytes: &[u8]) {
        let kept = &bytes[..bytes.len().min(self.room)];
        self.room -= kept.len();
        if let Some(file) = &mut self.file
            && (file.write_all(kept).is_err() || self.room == 0)
        {
            self.file = None;
        }
    }

    fn create(id: SessionId) -> Option<File> {
        let directory = PathBuf::from(std::env::var_os(DUMP_VARIABLE)?);
        create_folder(&directory).ok()?;
        create_file(&directory.join(format!("session-{}-output.bin", id.0))).ok()
    }
}
