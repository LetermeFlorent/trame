use std::env;
use std::path::PathBuf;

use super::{FontSource, SystemFonts, existing, first_family};

const SYSTEM_DIRS: &[&str] = &[
    "/System/Library/Fonts",
    "/System/Library/Fonts/Supplemental",
    "/Library/Fonts",
];

const TERMINAL: &[&[&str]] = &[
    &["SFNSMono.ttf", "SFNSMonoItalic.ttf"],
    &["Menlo.ttc"],
    &["Monaco.ttf"],
];

const INTERFACE: &[&[&str]] = &[
    &["SFNS.ttf", "SFNSItalic.ttf"],
    &["HelveticaNeue.ttc"],
    &["Helvetica.ttc"],
];

const EMOJI: &[&str] = &["Apple Color Emoji.ttc"];

const FALLBACK: &[&str] = &[
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

pub fn system_fonts() -> SystemFonts {
    let dirs = font_dirs();
    SystemFonts {
        terminal: first_family(&dirs, TERMINAL),
        interface: first_family(&dirs, INTERFACE),
        emoji: existing(&dirs, EMOJI),
        fallback: existing(&dirs, FALLBACK),
    }
}

pub fn fallback_for(_: char, _: bool) -> Option<FontSource> {
    None
}

fn font_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library").join("Fonts"))
        .into_iter()
        .collect();
    dirs.extend(SYSTEM_DIRS.iter().map(PathBuf::from));
    dirs
}
