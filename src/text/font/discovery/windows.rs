use std::env;
use std::path::PathBuf;

use super::{FontSource, SystemFonts, existing, first_family};

const TERMINAL: &[&[&str]] = &[
    &["CascadiaMono.ttf", "CascadiaMonoItalic.ttf"],
    &[
        "consola.ttf",
        "consolab.ttf",
        "consolai.ttf",
        "consolaz.ttf",
    ],
    &["cour.ttf", "courbd.ttf", "couri.ttf", "courbi.ttf"],
];

const INTERFACE: &[&[&str]] = &[
    &["segoeui.ttf", "seguisb.ttf", "segoeuib.ttf", "segoeuii.ttf"],
    &["tahoma.ttf", "tahomabd.ttf"],
    &["arial.ttf", "arialbd.ttf", "ariali.ttf"],
];

const EMOJI: &[&str] = &["seguiemj.ttf"];

const FALLBACK: &[&str] = &[
    "seguisym.ttf",
    "consola.ttf",
    "segoeui.ttf",
    "msyh.ttc",
    "YuGothM.ttc",
    "malgun.ttf",
    "Nirmala.ttc",
    "ebrima.ttf",
    "gadugi.ttf",
    "seguihis.ttf",
    "mmrtext.ttf",
    "himalaya.ttf",
    "taile.ttf",
    "ntailu.ttf",
    "phagspa.ttf",
    "monbaiti.ttf",
    "msyi.ttf",
    "javatext.ttf",
    "simsun.ttc",
    "l_10646.ttf",
    "arial.ttf",
    "times.ttf",
    "cambria.ttc",
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
    let mut dirs = Vec::new();
    if let Some(local) = env::var_os("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(local)
                .join("Microsoft")
                .join("Windows")
                .join("Fonts"),
        );
    }
    if let Some(root) = env::var_os("SystemRoot").or_else(|| env::var_os("windir")) {
        dirs.push(PathBuf::from(root).join("Fonts"));
    }
    dirs
}
