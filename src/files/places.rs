use std::env;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

const HOME: &str = "~";

pub fn home_dir() -> Option<PathBuf> {
    let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    env::var_os(key)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

pub fn short_path(path: &Path, home: Option<&Path>) -> String {
    let inside = home.and_then(|home| path.strip_prefix(home).ok());
    match inside {
        Some(rest) if rest.as_os_str().is_empty() => HOME.to_owned(),
        Some(rest) => format!("{HOME}{MAIN_SEPARATOR}{}", rest.display()),
        None => path.display().to_string(),
    }
}
