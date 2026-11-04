use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use png::Decoder;
use trame::files::{
    CAPTURE_MAX_AGE, capture_name, captures_folder, encode_png, join_paths, remove_old_captures,
};

use crate::Sandbox;

#[test]
fn paths_are_joined_by_a_space_and_quoted_when_they_hold_one() {
    let paths = [
        PathBuf::from("C:/data/a.txt"),
        PathBuf::from("C:/my files/b.txt"),
    ];
    assert_eq!(join_paths(&paths), "C:/data/a.txt \"C:/my files/b.txt\"");
    assert_eq!(join_paths(&[]), "");
}

#[test]
fn a_capture_is_named_after_its_unix_milliseconds() {
    assert_eq!(capture_name(1_762_000_000_123), "capture-1762000000123.png");
}

#[test]
fn captures_live_in_the_temporary_folder() {
    assert!(captures_folder().starts_with(std::env::temp_dir()));
    assert!(captures_folder().ends_with("trame/captures"));
}

#[test]
fn only_files_older_than_the_limit_are_removed() {
    let sandbox = Sandbox::new("captures");
    let old = sandbox.root.join("capture-1.png");
    fs::write(&old, "x").expect("written");
    remove_old_captures(&sandbox.root, SystemTime::now(), CAPTURE_MAX_AGE);
    assert!(old.exists());
    let later = SystemTime::now() + CAPTURE_MAX_AGE + Duration::from_secs(60);
    remove_old_captures(&sandbox.root, later, CAPTURE_MAX_AGE);
    assert!(!old.exists());
    remove_old_captures(&sandbox.root.join("missing"), later, CAPTURE_MAX_AGE);
}

#[test]
fn encoded_pixels_decode_back_unchanged() {
    let pixels: Vec<u8> = (0..2 * 3 * 4).map(|n| (n * 9) as u8).collect();
    let bytes = encode_png(2, 3, &pixels).expect("encoded");
    let mut reader = Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .expect("a png");
    let mut decoded = vec![0; reader.output_buffer_size().expect("a size")];
    let info = reader.next_frame(&mut decoded).expect("a frame");
    assert_eq!((info.width, info.height), (2, 3));
    assert_eq!(decoded, pixels);
}

#[test]
fn a_buffer_of_the_wrong_size_is_refused() {
    assert!(encode_png(2, 2, &[0; 5]).is_err());
}
