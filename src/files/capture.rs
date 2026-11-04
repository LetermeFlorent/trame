use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use png::{BitDepth, ColorType, Compression, Encoder};

use super::{create_folder, write_atomic};

pub const CAPTURE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

pub fn quote_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    if text.contains(' ') {
        format!("\"{text}\"")
    } else {
        text.into_owned()
    }
}

pub fn join_paths(paths: &[PathBuf]) -> String {
    let quoted: Vec<String> = paths.iter().map(|path| quote_path(path)).collect();
    quoted.join(" ")
}

pub fn capture_name(millis: u128) -> String {
    format!("capture-{millis}.png")
}

pub fn captures_folder() -> PathBuf {
    std::env::temp_dir().join("trame").join("captures")
}

pub fn remove_old_captures(folder: &Path, now: SystemTime, max_age: Duration) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let age = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok());
        if age.is_some_and(|age| age > max_age) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

pub fn encode_png(width: usize, height: usize, rgba: &[u8]) -> io::Result<Vec<u8>> {
    let size = |value: usize| u32::try_from(value).map_err(io::Error::other);
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, size(width)?, size(height)?);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_compression(Compression::Fast);
    let mut writer = encoder.write_header().map_err(io::Error::other)?;
    writer.write_image_data(rgba).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
    Ok(bytes)
}

pub fn save_capture(width: usize, height: usize, rgba: &[u8]) -> io::Result<String> {
    let bytes = encode_png(width, height, rgba)?;
    let folder = captures_folder();
    create_folder(&folder)?;
    let now = SystemTime::now();
    let millis = now
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_millis();
    let path = folder.join(capture_name(millis));
    write_atomic(&path, &bytes)?;
    remove_old_captures(&folder, now, CAPTURE_MAX_AGE);
    Ok(quote_path(&path))
}
