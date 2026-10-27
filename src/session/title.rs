use std::path::{Path, PathBuf};

use super::Session;
use super::pty::Pty;

impl Session {
    pub fn label(&self) -> String {
        let title = self
            .title()
            .map(|title| title.trim().to_owned())
            .filter(|title| !title.is_empty() && !names_shell(title, &self.shell));
        if let Some(title) = title {
            return title;
        }
        self.cwd()
            .as_deref()
            .and_then(folder_name)
            .unwrap_or_else(|| self.shell.clone())
    }

    pub fn cwd(&self) -> Option<PathBuf> {
        self.pty
            .as_ref()
            .and_then(Pty::cwd)
            .or_else(|| self.terminal().screen().cwd().map(PathBuf::from))
    }

    pub fn title(&self) -> Option<String> {
        self.terminal().screen().title().map(str::to_owned)
    }
}

pub(super) fn shell_name(program: &Path) -> String {
    program
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| program.to_string_lossy().into_owned())
}

fn names_shell(title: &str, shell: &str) -> bool {
    Path::new(title)
        .file_stem()
        .is_some_and(|stem| stem.to_string_lossy().eq_ignore_ascii_case(shell))
}

fn folder_name(path: &Path) -> Option<String> {
    let name = match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => path.to_string_lossy().into_owned(),
    };
    (!name.is_empty()).then_some(name)
}

pub fn starting_label(cwd: Option<&Path>, program: &Path) -> String {
    cwd.and_then(folder_name)
        .unwrap_or_else(|| shell_name(program))
}
