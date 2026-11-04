use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use trame::files::LogFile;

use super::Sandbox;

const LOG_NAME: &str = "trame.log";
const ROOMY: u64 = 4096;
const TINY: u64 = 100;

fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("the log is read")
}

#[test]
fn an_error_reaches_the_log_file() {
    let sandbox = Sandbox::new("log-error");
    let path = sandbox.root.join(LOG_NAME);
    LogFile::new(path.clone(), ROOMY)
        .append(at(1_793_179_805), "the session could not be saved")
        .expect("the line is written");
    assert_eq!(
        read(&path),
        "2026-10-28T09:30:05Z the session could not be saved\n"
    );
}

#[test]
fn later_errors_are_added_below() {
    let sandbox = Sandbox::new("log-order");
    let path = sandbox.root.join(LOG_NAME);
    let log = LogFile::new(path.clone(), ROOMY);
    log.append(at(0), "first").expect("the line is written");
    log.append(at(61), "second").expect("the line is written");
    assert_eq!(
        read(&path),
        "1970-01-01T00:00:00Z first\n1970-01-01T00:01:01Z second\n"
    );
}

#[test]
fn a_leap_day_is_stamped_in_utc() {
    let sandbox = Sandbox::new("log-leap");
    let path = sandbox.root.join(LOG_NAME);
    LogFile::new(path.clone(), ROOMY)
        .append(at(951_868_799), "late")
        .expect("the line is written");
    assert_eq!(read(&path), "2000-02-29T23:59:59Z late\n");
}

#[test]
fn a_missing_state_folder_is_created() {
    let sandbox = Sandbox::new("log-folder");
    let path = sandbox.root.join("state").join(LOG_NAME);
    LogFile::new(path.clone(), ROOMY)
        .append(at(0), "first")
        .expect("the line is written");
    assert_eq!(read(&path), "1970-01-01T00:00:00Z first\n");
}

#[test]
fn a_full_log_starts_over_and_keeps_one_previous_file() {
    let sandbox = Sandbox::new("log-full");
    let path = sandbox.root.join(LOG_NAME);
    let log = LogFile::new(path.clone(), TINY);
    for index in 0..20 {
        log.append(at(0), &format!("error {index}"))
            .expect("the line is written");
    }
    assert!(read(&path).ends_with("error 19\n"));
    assert!(fs::metadata(&path).expect("the log exists").len() < 2 * TINY);
    assert!(read(&sandbox.root.join("trame.log.old")).contains("error 15\n"));
    let files = fs::read_dir(&sandbox.root).expect("the sandbox is listed");
    assert_eq!(files.count(), 2);
}

#[cfg(unix)]
#[test]
fn only_the_owner_reads_the_log() {
    let sandbox = Sandbox::new("log-mode");
    let path = sandbox.root.join(LOG_NAME);
    LogFile::new(path.clone(), ROOMY)
        .append(at(0), "private")
        .expect("the line is written");
    let mode = fs::metadata(&path).expect("the log exists").permissions();
    assert_eq!(mode.mode() & 0o777, 0o600);
}
