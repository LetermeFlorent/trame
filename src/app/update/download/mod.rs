use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::RELEASES;
use super::version::Version;

mod progress;

pub type Progress<'a> = Option<&'a dyn Fn(u8)>;

const CHECK_SECONDS: &str = "1";
const DOWNLOAD_SECONDS: &str = "120";
const SLOWEST_BYTES_PER_SECOND: &str = "10000";
const STALL_SECONDS: &str = "10";

#[cfg(windows)]
const NULL_DEVICE: &str = "NUL";
#[cfg(not(windows))]
const NULL_DEVICE: &str = "/dev/null";

pub fn latest() -> Option<Version> {
    let output = curl()
        .args([
            "--silent",
            "--max-time",
            CHECK_SECONDS,
            "--output",
            NULL_DEVICE,
        ])
        .args(["--write-out", "%{redirect_url}"])
        .arg(format!("{RELEASES}/latest"))
        .output()
        .ok()?;
    Version::released(&String::from_utf8_lossy(&output.stdout))
}

pub fn download(version: Version, asset: &str, target: &Path, shown: Progress) -> io::Result<()> {
    let mut command = curl();
    if shown.is_none() {
        lower_priority(&mut command);
    }
    transfer(command, version, asset, target, shown.unwrap_or(&|_| {}))
}

fn transfer(
    mut command: Command,
    version: Version,
    asset: &str,
    target: &Path,
    on_percent: &dyn Fn(u8),
) -> io::Result<()> {
    let mut child = command
        .args(["--progress-bar", "--fail", "--location"])
        .args(["--max-time", DOWNLOAD_SECONDS])
        .args(["--speed-limit", SLOWEST_BYTES_PER_SECOND])
        .args(["--speed-time", STALL_SECONDS, "--output"])
        .arg(target)
        .arg(format!("{RELEASES}/download/{version}/{asset}"))
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(bar) = child.stderr.take() {
        progress::follow(bar, on_percent);
    }
    let status = child.wait()?;
    match status.success() {
        true => Ok(()),
        false => Err(io::Error::other(format!("the download stopped ({status})"))),
    }
}

fn curl() -> Command {
    let mut command = Command::new(curl_program());
    command
        .args(["--proto", "=https", "--proto-redir", "=https"])
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    hide_console(&mut command);
    command
}

#[cfg(windows)]
fn curl_program() -> PathBuf {
    std::env::var_os("SystemRoot").map_or_else(
        || PathBuf::from("curl.exe"),
        |root| PathBuf::from(root).join("System32").join("curl.exe"),
    )
}

#[cfg(not(windows))]
fn curl_program() -> PathBuf {
    PathBuf::from("curl")
}

#[cfg(windows)]
fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console(_: &mut Command) {}

#[cfg(windows)]
fn lower_priority(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::{BELOW_NORMAL_PRIORITY_CLASS, CREATE_NO_WINDOW};
    command.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
}

#[cfg(not(windows))]
fn lower_priority(_: &mut Command) {}
