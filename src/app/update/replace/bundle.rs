use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::super::download::{self, Progress};
use super::super::version::Version;
use super::swap;

const EXTENSION: &str = "app";
const CONTENTS: &str = "Contents";
const PROGRAMS: &str = "MacOS";
const ARCHIVE_SUFFIX: &str = ".zip";
const UNPACK_SUFFIX: &str = ".unzip";
const DITTO: &str = "/usr/bin/ditto";

pub fn root(exe: &Path) -> Option<PathBuf> {
    let programs = exe.parent()?;
    let contents = programs.parent()?;
    let bundle = contents.parent()?;
    let inside = programs.file_name()? == PROGRAMS && contents.file_name()? == CONTENTS;
    let named = bundle.extension()? == EXTENSION;
    (inside && named).then(|| bundle.to_path_buf())
}

pub fn fetch(version: Version, asset: &str, part: &Path, shown: Progress) -> io::Result<()> {
    let archive = swap::sibling(part, ARCHIVE_SUFFIX);
    let fetched =
        download::download(version, asset, &archive, shown).and_then(|()| unpack(&archive, part));
    swap::discard_partial(&archive, "download");
    fetched
}

fn unpack(archive: &Path, part: &Path) -> io::Result<()> {
    let folder = swap::sibling(part, UNPACK_SUFFIX);
    let unpacked = extract(archive, &folder)
        .and_then(|()| application_in(&folder))
        .and_then(|app| fs::rename(app, part));
    swap::discard_partial(&folder, "unpacked update");
    unpacked
}

fn extract(archive: &Path, folder: &Path) -> io::Result<()> {
    let status = Command::new(DITTO)
        .args(["-x", "-k"])
        .arg(archive)
        .arg(folder)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    match status.success() {
        true => Ok(()),
        false => Err(io::Error::other(format!(
            "the archive did not open ({status})"
        ))),
    }
}

fn application_in(folder: &Path) -> io::Result<PathBuf> {
    fs::read_dir(folder)?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == EXTENSION)
        })
        .ok_or_else(|| io::Error::other("the archive holds no application"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_application_folder_is_found_from_its_program() {
        let exe = Path::new("/Applications/trame.app/Contents/MacOS/trame");
        assert_eq!(root(exe), Some(PathBuf::from("/Applications/trame.app")));
    }

    #[test]
    fn a_program_outside_an_application_folder_has_none() {
        assert_eq!(root(Path::new("/usr/bin/trame")), None);
        assert_eq!(root(Path::new("/opt/trame/Contents/MacOS/trame")), None);
        assert_eq!(root(Path::new("C:/Programs/trame/trame.exe")), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn an_archive_made_by_the_release_opens_into_the_new_application() {
        let scratch = std::env::temp_dir().join(format!("trame-bundle-{}", std::process::id()));
        let program = scratch.join("build/trame.app/Contents/MacOS/trame");
        fs::create_dir_all(program.parent().expect("a folder")).expect("the build");
        let mut bytes = b"\xca\xfe\xba\xbe".to_vec();
        bytes.resize(1024 * 1024, 0);
        fs::write(&program, bytes).expect("the program");
        let archive = scratch.join("trame-macos.zip");
        let zipped = Command::new(DITTO)
            .args(["-c", "-k", "--keepParent"])
            .arg(scratch.join("build/trame.app"))
            .arg(&archive)
            .status()
            .expect("ditto runs");
        assert!(zipped.success());
        let part = scratch.join("trame.app.9.9.9.ready.part");
        unpack(&archive, &part).expect("the archive opens");
        assert!(super::super::check::release(&part).is_ok());
        assert!(!swap::sibling(&part, UNPACK_SUFFIX).exists());
        fs::remove_dir_all(&scratch).expect("cleanup");
    }
}
