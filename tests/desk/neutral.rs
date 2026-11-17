use std::fs;
use std::path::{Path, PathBuf};

const DESK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/desk");
const RUNNING: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/app/running");
const FOREIGN: [&str; 8] = [
    "winit",
    "wgpu",
    "arboard",
    "crate::ui",
    "crate::render",
    "crate::text",
    "gtk",
    "adw",
];
const MAX_LINES: usize = 150;

fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).expect("a source folder") {
        let path = entry.expect("a source entry").path();
        if path.is_dir() {
            found.extend(sources(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
    found
}

fn text(path: &Path) -> String {
    fs::read_to_string(path).expect("a readable source")
}

#[test]
fn the_desk_names_no_front_end() {
    for path in sources(Path::new(DESK)) {
        for (number, line) in text(&path).lines().enumerate() {
            let named = FOREIGN.iter().find(|word| line.contains(*word));
            assert!(
                named.is_none(),
                "{}:{} names {named:?}",
                path.display(),
                number + 1
            );
        }
    }
}

#[test]
fn desk_and_running_files_stay_short() {
    let all = [DESK, RUNNING].map(|dir| sources(Path::new(dir)));
    for path in all.iter().flatten() {
        let lines = text(path).lines().count();
        assert!(lines <= MAX_LINES, "{} has {lines} lines", path.display());
    }
}
