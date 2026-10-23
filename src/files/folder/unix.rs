use std::path::PathBuf;
use std::process::Command;

use super::FolderRequest;

#[cfg(target_os = "macos")]
fn command(request: &FolderRequest) -> Command {
    let script = format!(
        "POSIX path of (choose folder with prompt {:?})",
        request.title
    );
    let mut command = Command::new("osascript");
    command.arg("-e").arg(script);
    command
}

#[cfg(not(target_os = "macos"))]
fn command(request: &FolderRequest) -> Command {
    let mut command = Command::new("zenity");
    command
        .args(["--file-selection", "--directory", "--title"])
        .arg(&request.title);
    if let Some(start) = &request.start {
        command.arg("--filename").arg(start.join(""));
    }
    command
}

pub(super) fn choose(request: &FolderRequest) -> Option<PathBuf> {
    let output = command(request)
        .output()
        .inspect_err(|error| eprintln!("trame: the folder window could not open: {error}"))
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let folder = text.trim_end_matches(['\r', '\n']);
    (!folder.is_empty()).then(|| PathBuf::from(folder))
}
