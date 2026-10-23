use std::env;
use std::path::PathBuf;

const SYSTEM_DIRS: &[&str] = &[
    "/System/Library/Fonts",
    "/System/Library/Fonts/Supplemental",
    "/Library/Fonts",
];

pub(super) const TERMINAL: &[&[&str]] = &[
    &["SFNSMono.ttf", "SFNSMonoItalic.ttf"],
    &["Menlo.ttc"],
    &["Monaco.ttf"],
];

pub(super) const INTERFACE: &[&[&str]] = &[
    &["SFNS.ttf", "SFNSItalic.ttf"],
    &["HelveticaNeue.ttc"],
    &["Helvetica.ttc"],
];

pub(super) const EMOJI: &[&str] = &["Apple Color Emoji.ttc"];

pub(super) const FALLBACK: &[&str] = &[
    "Menlo.ttc",
    "Apple Symbols.ttf",
    "SFNS.ttf",
    "PingFang.ttc",
    "Hiragino Sans GB.ttc",
    "AppleSDGothicNeo.ttc",
    "Kohinoor.ttc",
    "Arial Unicode.ttf",
    "STIXTwoMath.otf",
    "Geneva.ttf",
];

pub(super) fn font_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library").join("Fonts"))
        .into_iter()
        .collect();
    dirs.extend(SYSTEM_DIRS.iter().map(PathBuf::from));
    dirs
}
