use std::collections::HashSet;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::log_error;

const EXTENSION: &str = "vt";
const MAX_KEY_LENGTH: usize = 40;

pub(super) fn file_name(key: &str) -> String {
    format!("{key}.{EXTENSION}")
}

pub(super) fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= MAX_KEY_LENGTH
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub(super) fn remove_stale(folder: &Path, kept: &HashSet<String>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let stale = entry.file_type().is_ok_and(|kind| kind.is_file())
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| !kept.contains(name));
        if stale && let Err(error) = fs::remove_file(entry.path()) {
            report_unless_missing("deleted", &error);
        }
    }
}

pub(super) fn report_unless_missing(action: &str, error: &io::Error) {
    if error.kind() != ErrorKind::NotFound {
        log_error!("a saved tab text could not be {action}: {error}");
    }
}
