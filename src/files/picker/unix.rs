use std::path::PathBuf;
use std::process::Command;

use super::{PickKind, PickRequest};
use crate::log_error;

#[cfg(target_os = "macos")]
pub(super) fn choose(request: &PickRequest, kind: PickKind) -> Option<PathBuf> {
    run(osascript(request, kind))
}

#[cfg(not(target_os = "macos"))]
pub(super) fn choose(request: &PickRequest, kind: PickKind) -> Option<PathBuf> {
    use super::portal::{self, Answer};
    match portal::choose(request, kind) {
        Ok(Answer::Picked(path)) => Some(path),
        Ok(Answer::Cancelled) => None,
        Err(error) => {
            log_error!("the file portal is not available ({error}), trying zenity");
            run(zenity(request, kind))
        }
    }
}

#[cfg(target_os = "macos")]
fn osascript(request: &PickRequest, kind: PickKind) -> Command {
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
fn zenity(request: &PickRequest, kind: PickKind) -> Command {
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

fn run(mut command: Command) -> Option<PathBuf> {
    let output = command
        .output()
        .inspect_err(|error| log_error!("the file window could not open: {error}"))
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let path = text.trim_end_matches(['\r', '\n']);
    (!path.is_empty()).then(|| PathBuf::from(path))
}
