use super::host::is_this_machine;
use super::invisible::is_invisible_format;

const DRIVE_PREFIX_LEN: usize = 3;
const PERCENT_HEX_DIGITS: usize = 2;
const PERCENT_ESCAPE_LEN: usize = PERCENT_HEX_DIGITS + 1;
const HEX_RADIX: u32 = 16;

pub(crate) fn file_url_path(url: &[u8]) -> Option<String> {
    let rest = url.strip_prefix(b"file://")?;
    let path_start = rest.iter().position(|&byte| byte == b'/')?;
    let (host, path) = rest.split_at(path_start);
    if !is_this_machine(host) {
        return None;
    }
    let path = String::from_utf8(percent_decode(path)).ok()?;
    local_folder(strip_drive_slash(path))
}

pub(crate) fn local_folder(path: String) -> Option<String> {
    let bytes = path.as_bytes();
    let from_drive = bytes.len() >= DRIVE_PREFIX_LEN
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && is_separator(bytes[2]);
    let from_root =
        bytes.first() == Some(&b'/') && !matches!(bytes.get(1), Some(b'/' | b'\\' | b'?'));
    let printable = !path
        .chars()
        .any(|c| c.is_control() || is_invisible_format(c));
    ((from_drive || from_root) && printable).then_some(path)
}

fn is_separator(byte: u8) -> bool {
    matches!(byte, b'/' | b'\\')
}

fn percent_decode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'%' && index + PERCENT_HEX_DIGITS < input.len() {
            let hex = std::str::from_utf8(&input[index + 1..index + PERCENT_ESCAPE_LEN]).ok();
            if let Some(value) = hex.and_then(|hex| u8::from_str_radix(hex, HEX_RADIX).ok()) {
                output.push(value);
                index += PERCENT_ESCAPE_LEN;
                continue;
            }
        }
        output.push(input[index]);
        index += 1;
    }
    output
}

fn strip_drive_slash(path: String) -> String {
    let bytes = path.as_bytes();
    let has_drive = bytes.len() >= DRIVE_PREFIX_LEN
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && bytes[2] == b':';
    if has_drive {
        path[1..].to_string()
    } else {
        path
    }
}
