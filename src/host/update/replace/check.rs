use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub const SMALLEST_RELEASE: u64 = 1024 * 1024;
pub const BUNDLE_PROGRAM: [&str; 3] = ["Contents", "MacOS", "trame"];
const MAGIC_LENGTH: u64 = 8;
pub const MAGICS: [&[u8]; 6] = [
    b"MZ",
    b"\x7fELF",
    b"\xca\xfe\xba\xbe",
    b"\xcf\xfa\xed\xfe",
    b"!<arch>\n",
    b"\xed\xab\xee\xdb",
];

pub fn release(path: &Path) -> io::Result<()> {
    let mut file = File::open(program(path))?;
    let mut start = Vec::new();
    file.by_ref().take(MAGIC_LENGTH).read_to_end(&mut start)?;
    let large_enough = file.metadata()?.len() >= SMALLEST_RELEASE;
    let known = MAGICS.iter().any(|magic| start.starts_with(magic));
    match large_enough && known {
        true => Ok(()),
        false => Err(io::Error::other("the download is not a trame release")),
    }
}

fn program(path: &Path) -> PathBuf {
    match path.is_dir() {
        true => BUNDLE_PROGRAM
            .iter()
            .fold(path.to_path_buf(), |path, part| path.join(part)),
        false => path.to_path_buf(),
    }
}
