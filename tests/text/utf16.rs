use std::ffi::OsStr;

use trame::utf16::wide;

#[test]
fn wide_ends_with_a_nul() {
    assert_eq!(wide(OsStr::new("")), vec![0]);
    assert_eq!(wide(OsStr::new("ab")), vec![97, 98, 0]);
}

#[test]
fn wide_encodes_characters_outside_the_basic_plane_as_pairs() {
    let units = wide(OsStr::new("\u{1F600}"));
    assert_eq!(units, vec![0xD83D, 0xDE00, 0]);
}
