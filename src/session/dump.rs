#![cfg_attr(not(feature = "dump"), allow(dead_code))]

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::SessionId;
use super::pty::DUMP_VARIABLE;
use crate::files::{create_file, create_folder};

const MAX_FILE_BYTES: usize = 64 * 1024 * 1024;

pub struct Dump {
    file: Option<File>,
    reads: Option<File>,
    room: usize,
    reads_room: usize,
}

impl Dump {
    pub fn open(id: SessionId) -> Self {
        let directory = std::env::var_os(DUMP_VARIABLE).map(PathBuf::from);
        Self::in_folder(directory.as_deref(), id, MAX_FILE_BYTES)
    }

    fn in_folder(directory: Option<&Path>, id: SessionId, room: usize) -> Self {
        Self {
            file: Self::create(directory, id, "output.bin"),
            reads: Self::create(directory, id, "reads.log"),
            room,
            reads_room: room,
        }
    }

    pub fn record(&mut self, bytes: &[u8], working: bool) {
        let kept = &bytes[..bytes.len().min(self.room)];
        self.room -= kept.len();
        if let Some(file) = &mut self.file
            && (file.write_all(kept).is_err() || self.room == 0)
        {
            self.file = None;
        }
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let line = format!("{} {} {}\n", at.as_millis(), bytes.len(), u8::from(working));
        if let Some(reads) = &mut self.reads {
            let fits = line.len() <= self.reads_room;
            if !fits || reads.write_all(line.as_bytes()).is_err() {
                self.reads = None;
            }
            self.reads_room = self.reads_room.saturating_sub(line.len());
        }
    }

    fn create(directory: Option<&Path>, id: SessionId, kind: &str) -> Option<File> {
        let directory = directory?;
        create_folder(directory).ok()?;
        create_file(&directory.join(format!("session-{}-{kind}", id.0))).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reads_log_stops_at_the_same_cap_as_the_output_file() {
        let directory = std::env::temp_dir().join(format!("trame-dump-{}", std::process::id()));
        let mut dump = Dump::in_folder(Some(&directory), SessionId(7), 100);
        for _ in 0..50 {
            dump.record(b"0123456789", true);
        }
        let size = |kind: &str| {
            std::fs::metadata(directory.join(format!("session-7-{kind}")))
                .unwrap()
                .len()
        };
        assert_eq!(size("output.bin"), 100);
        assert!(
            (1..=100).contains(&size("reads.log")),
            "{}",
            size("reads.log")
        );
        std::fs::remove_dir_all(&directory).unwrap();
    }
}
