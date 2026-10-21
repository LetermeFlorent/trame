#[cfg(all(unix, not(target_os = "macos")))]
#[path = "fontconfig.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform;
#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontSource {
    pub path: PathBuf,
    pub index: Option<u32>,
}

impl FontSource {
    pub fn file(path: PathBuf) -> Self {
        Self { path, index: None }
    }
}

#[derive(Debug, Default)]
pub struct SystemFonts {
    pub terminal: Vec<FontSource>,
    pub interface: Vec<FontSource>,
    pub emoji: Vec<FontSource>,
    pub fallback: Vec<FontSource>,
}

pub fn system_fonts() -> SystemFonts {
    platform::system_fonts()
}

pub fn fallback_for(ch: char, emoji: bool) -> Option<FontSource> {
    platform::fallback_for(ch, emoji)
}

#[cfg(any(windows, target_os = "macos"))]
fn locate(dirs: &[PathBuf], name: &str) -> Option<FontSource> {
    dirs.iter()
        .map(|dir| dir.join(name))
        .find(|path| is_file(path))
        .map(FontSource::file)
}

#[cfg(any(windows, target_os = "macos"))]
fn existing(dirs: &[PathBuf], names: &[&str]) -> Vec<FontSource> {
    names.iter().filter_map(|name| locate(dirs, name)).collect()
}

#[cfg(any(windows, target_os = "macos"))]
fn first_family(dirs: &[PathBuf], families: &[&[&str]]) -> Vec<FontSource> {
    families
        .iter()
        .find(|names| {
            names
                .first()
                .is_some_and(|name| locate(dirs, name).is_some())
        })
        .map(|names| existing(dirs, names))
        .unwrap_or_default()
}

fn is_file(path: &Path) -> bool {
    path.metadata().is_ok_and(|metadata| metadata.is_file())
}
