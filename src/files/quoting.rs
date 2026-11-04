use std::path::{Path, PathBuf};

pub fn quote_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    if text.contains(' ') {
        format!("\"{text}\"")
    } else {
        text.into_owned()
    }
}

pub fn join_paths(paths: &[PathBuf]) -> String {
    let quoted: Vec<String> = paths.iter().map(|path| quote_path(path)).collect();
    quoted.join(" ")
}
