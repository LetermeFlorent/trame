use std::path::PathBuf;
use std::process::Command;

use super::{PickKind, PickRequest};

#[cfg(target_os = "macos")]
fn command(request: &PickRequest, kind: PickKind) -> Command {
    let chooser = match kind {
        PickKind::Folder => "choose folder",
        PickKind::Image => "choose file of type {\"public.png\"}",
    };
    let script = format!("POSIX path of ({chooser} with prompt {:?})", request.title);
    let mut command = Command::new("osascript");
    command.arg("-e").arg(script);
    command
}

#[cfg(not(target_os = "macos"))]
fn command(request: &PickRequest, kind: PickKind) -> Command {
    let mut command = Command::new("zenity");
    command.arg("--file-selection");
    match kind {
        PickKind::Folder => command.arg("--directory"),
        PickKind::Image => command.arg("--file-filter=PNG | *.png"),
    };
    command.arg("--title").arg(&request.title);
    if let Some(start) = &request.start {
        command.arg("--filename").arg(start.join(""));
    }
    command
}

pub(super) fn choose(request: &PickRequest, kind: PickKind) -> Option<PathBuf> {
    let output = command(request, kind)
        .output()
        .inspect_err(|error| eprintln!("trame: the file window could not open: {error}"))
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let path = text.trim_end_matches(['\r', '\n']);
    (!path.is_empty()).then(|| PathBuf::from(path))
}
