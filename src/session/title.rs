use std::path::Path;

use super::Session;

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

    pub fn shell_name(&self) -> &str {
        &self.shell
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
