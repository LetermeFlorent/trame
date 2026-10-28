mod stamp;

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::SystemTime;

use super::{append_file, create_folder};
use stamp::stamp;

const LOG_LIMIT: u64 = 256 * 1024;
const PREVIOUS_SUFFIX: &str = ".old";

static LOG: OnceLock<Mutex<LogFile>> = OnceLock::new();

#[macro_export]
macro_rules! log_error {
    ($($message:tt)*) => {
        $crate::files::record_error(::std::format_args!($($message)*))
    };
}

pub struct LogFile {
    path: PathBuf,
    limit: u64,
}

impl LogFile {
    pub fn new(path: PathBuf, limit: u64) -> Self {
        Self { path, limit }
    }

    pub fn append(&self, at: SystemTime, message: &str) -> io::Result<()> {
        if let Some(folder) = self.path.parent() {
            create_folder(folder)?;
        }
        if self.is_full() {
            self.start_over()?;
        }
        let line = format!("{} {message}\n", stamp(at));
        append_file(&self.path)?.write_all(line.as_bytes())
    }

    fn is_full(&self) -> bool {
        fs::metadata(&self.path).is_ok_and(|metadata| metadata.len() >= self.limit)
    }

    fn start_over(&self) -> io::Result<()> {
        let mut previous = self.path.as_os_str().to_owned();
        previous.push(PREVIOUS_SUFFIX);
        match fs::rename(&self.path, PathBuf::from(previous)) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }
}

pub fn start_log(path: PathBuf) {
    let _ = LOG.set(Mutex::new(LogFile::new(path, LOG_LIMIT)));
}

pub fn record_error(message: fmt::Arguments<'_>) {
    let message = message.to_string();
    eprintln!("trame: {message}");
    let Some(log) = LOG.get() else {
        return;
    };
    let log = log.lock().unwrap_or_else(PoisonError::into_inner);
    if let Err(error) = log.append(SystemTime::now(), &message) {
        eprintln!("trame: the log file could not be written: {error}");
    }
}
