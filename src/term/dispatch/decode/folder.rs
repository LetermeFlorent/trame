use super::host::is_this_machine;

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
    let from_drive = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && is_separator(bytes[2]);
    let from_root =
        bytes.first() == Some(&b'/') && !matches!(bytes.get(1), Some(b'/' | b'\\' | b'?'));
    let printable = !path.chars().any(char::is_control);
    ((from_drive || from_root) && printable).then_some(path)
}

fn is_separator(byte: u8) -> bool {
    matches!(byte, b'/' | b'\\')
}

fn percent_decode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'%' && index + 2 < input.len() {
            let hex = std::str::from_utf8(&input[index + 1..index + 3]).ok();
            if let Some(value) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                output.push(value);
                index += 3;
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
    let has_drive =
        bytes.len() >= 3 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':';
    if has_drive {
        path[1..].to_string()
    } else {
        path
    }
}
