use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use super::SessionId;

const DIRECTORY_VARIABLE: &str = "TRAME_DUMP";

pub struct Dump(Option<File>);

impl Dump {
    pub fn open(id: SessionId) -> Self {
        Self(Self::create(id))
    }

    pub fn record(&mut self, bytes: &[u8]) {
        if let Some(file) = &mut self.0
            && file.write_all(bytes).is_err()
        {
            self.0 = None;
        }
    }

    fn create(id: SessionId) -> Option<File> {
        let directory = PathBuf::from(std::env::var_os(DIRECTORY_VARIABLE)?);
        fs::create_dir_all(&directory).ok()?;
        File::create(directory.join(format!("session-{}-output.bin", id.0))).ok()
    }
}
