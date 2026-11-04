mod folder;
mod host;
mod invisible;

pub(crate) use folder::{file_url_path, local_folder};
pub(crate) use invisible::is_invisible_format;

const SYMBOLS_PER_GROUP: usize = 4;
const BYTES_PER_GROUP: usize = 3;
const BITS_PER_SYMBOL: u32 = 6;
const BITS_PER_BYTE: u32 = 8;
const LOWERCASE_START: u8 = 26;
const DIGIT_START: u8 = 52;
const PLUS_VALUE: u8 = 62;
const SLASH_VALUE: u8 = 63;

pub(crate) fn decode_base64(input: &[u8]) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(input.len() / SYMBOLS_PER_GROUP * BYTES_PER_GROUP);
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for &byte in input {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + LOWERCASE_START,
            b'0'..=b'9' => byte - b'0' + DIGIT_START,
            b'+' | b'-' => PLUS_VALUE,
            b'/' | b'_' => SLASH_VALUE,
            b'=' | b'\r' | b'\n' => continue,
            _ => return None,
        };
        buffer = (buffer << BITS_PER_SYMBOL) | u32::from(value);
        bits += BITS_PER_SYMBOL;
        if bits >= BITS_PER_BYTE {
            bits -= BITS_PER_BYTE;
            output.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(output)
}
