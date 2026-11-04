mod launch;
mod runnable;

use std::fs;
use std::path::{Path, PathBuf};

use crate::term::file_url_path;
pub use launch::open_link;
use runnable::is_runnable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    Web(String),
    Path(String),
    Uri(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opening {
    Browser(String),
    File(PathBuf),
    Folder(PathBuf),
}

pub fn plan_opening(
    target: &LinkTarget,
    cwd: Option<&Path>,
    home: Option<&Path>,
) -> Option<Opening> {
    match target {
        LinkTarget::Web(url) => web(url),
        LinkTarget::Uri(uri) => web(uri).or_else(|| {
            let path = file_url_path(uri.as_bytes())?;
            local(Path::new(&path))
        }),
        LinkTarget::Path(path) => local(&resolve(path, cwd, home)?),
    }
}

fn web(url: &str) -> Option<Opening> {
    let (scheme, rest) = url.split_once("://")?;
    let known = ["http", "https"]
        .iter()
        .any(|allowed| scheme.eq_ignore_ascii_case(allowed));
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let clean = !url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '"' | '<' | '>' | '`'));
    (known && clean && !host.is_empty()).then(|| Opening::Browser(url.to_owned()))
}

fn resolve(text: &str, cwd: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(rest) = text.strip_prefix("~/").or_else(|| text.strip_prefix("~\\")) {
        return Some(home?.join(rest));
    }
    let path = Path::new(text);
    if path.is_absolute() {
        return Some(path.to_path_buf());
    }
    Some(cwd.filter(|cwd| cwd.is_absolute())?.join(path))
}

fn local(path: &Path) -> Option<Opening> {
    if is_share(path) {
        return None;
    }
    let path = plain(fs::canonicalize(path).ok()?);
    if is_share(&path) {
        return None;
    }
    let metadata = fs::metadata(&path).ok()?;
    if is_runnable(&path, &metadata) {
        return path
            .parent()
            .map(|folder| Opening::Folder(folder.to_path_buf()));
    }
    Some(match metadata.is_dir() {
        true => Opening::Folder(path),
        false => Opening::File(path),
    })
}

fn plain(path: PathBuf) -> PathBuf {
    let stripped = path
        .to_str()
        .and_then(|text| match text.strip_prefix(r"\\?\UNC\") {
            Some(share) => Some(format!(r"\\{share}")),
            None => text.strip_prefix(r"\\?\").map(str::to_owned),
        });
    stripped.map_or(path, PathBuf::from)
}

fn is_share(path: &Path) -> bool {
    let text = path.to_string_lossy();
    let local = text.starts_with(r"\\?\") && !text.starts_with(r"\\?\UNC\");
    !local && (text.starts_with(r"\\") || text.starts_with("//"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_are_told_apart_from_local_paths() {
        for share in [r"\\server\share", "//server/share", r"\\?\UNC\server\share"] {
            assert!(is_share(Path::new(share)), "{share}");
        }
        for local in [
            r"C:\dir\file.txt",
            r"\\?\C:\dir",
            "/usr/bin/env",
            "src/main.rs",
        ] {
            assert!(!is_share(Path::new(local)), "{local}");
        }
    }
}
