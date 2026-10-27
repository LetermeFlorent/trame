use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub fn wide(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(Some(0)).collect()
}
