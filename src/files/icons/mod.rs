mod codec;
mod resample;
mod store;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::write_atomic;
pub use resample::fit;
pub use store::IconStore;

pub const ICON_SIZE: u32 = 64;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn icon_path(dir: &Path, id: u64) -> PathBuf {
    dir.join(format!("{id:016x}.png"))
}

pub fn import_icon(source: &Path, dir: &Path) -> io::Result<u64> {
    let image = codec::decode(&fs::read(source)?)?;
    let pixels = fit(&image.pixels, image.width, image.height, ICON_SIZE);
    let id = fingerprint(&pixels);
    let path = icon_path(dir, id);
    if !path.exists() {
        write_atomic(&path, &codec::encode(&pixels, ICON_SIZE)?)?;
    }
    Ok(id)
}

pub fn load_icon(dir: &Path, id: u64) -> io::Result<Vec<u8>> {
    let image = codec::decode(&fs::read(icon_path(dir, id))?)?;
    Ok(fit(&image.pixels, image.width, image.height, ICON_SIZE))
}

fn fingerprint(pixels: &[u8]) -> u64 {
    pixels.iter().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}
