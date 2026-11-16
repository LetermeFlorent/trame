use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

const FILE_SCHEME: &str = "file://";

pub fn path_from_uri(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix(FILE_SCHEME)?;
    let path = &rest[rest.find('/')?..];
    let bytes = decode(path.as_bytes())?;
    Some(PathBuf::from(OsString::from_vec(bytes)))
}

fn decode(text: &[u8]) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        if text[index] == b'%' {
            let pair = text.get(index + 1..index + 3)?;
            let pair = std::str::from_utf8(pair).ok()?;
            bytes.push(u8::from_str_radix(pair, 16).ok()?);
            index += 3;
        } else {
            bytes.push(text[index]);
            index += 1;
        }
    }
    Some(bytes)
}
