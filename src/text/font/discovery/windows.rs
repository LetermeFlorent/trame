use std::env;
use std::path::PathBuf;

const CONSOLAS: &str = "consola.ttf";
const SEGOE_UI: &str = "segoeui.ttf";
const ARIAL: &str = "arial.ttf";
const FONTS_FOLDER: &str = "Fonts";

pub(super) const TERMINAL: &[&[&str]] = &[
    &["CascadiaMono.ttf", "CascadiaMonoItalic.ttf"],
    &[CONSOLAS, "consolab.ttf", "consolai.ttf", "consolaz.ttf"],
    &["cour.ttf", "courbd.ttf", "couri.ttf", "courbi.ttf"],
];

pub(super) const INTERFACE: &[&[&str]] = &[
    &[SEGOE_UI, "seguisb.ttf", "segoeuib.ttf", "segoeuii.ttf"],
    &["tahoma.ttf", "tahomabd.ttf"],
    &[ARIAL, "arialbd.ttf", "ariali.ttf"],
];

pub(super) const EMOJI: &[&str] = &["seguiemj.ttf"];

pub(super) const FALLBACK: &[&str] = &[
    "seguisym.ttf",
    CONSOLAS,
    SEGOE_UI,
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
    ARIAL,
    "times.ttf",
    "cambria.ttc",
];

pub(super) fn font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(local) = env::var_os("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(local)
                .join("Microsoft")
                .join("Windows")
                .join(FONTS_FOLDER),
        );
    }
    if let Some(root) = env::var_os("SystemRoot").or_else(|| env::var_os("windir")) {
        dirs.push(PathBuf::from(root).join(FONTS_FOLDER));
    }
    dirs
}
