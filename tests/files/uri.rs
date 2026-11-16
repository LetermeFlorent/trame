use std::path::PathBuf;

use trame::files::path_from_uri;

#[test]
fn a_file_uri_gives_its_path() {
    assert_eq!(
        path_from_uri("file:///home/me/dev"),
        Some(PathBuf::from("/home/me/dev"))
    );
}

#[test]
fn escaped_bytes_are_decoded() {
    assert_eq!(
        path_from_uri("file:///home/me/mes%20projets/%C3%A9t%C3%A9"),
        Some(PathBuf::from("/home/me/mes projets/été"))
    );
}

#[test]
fn a_host_before_the_path_is_skipped() {
    assert_eq!(
        path_from_uri("file://localhost/tmp"),
        Some(PathBuf::from("/tmp"))
    );
}

#[test]
fn other_schemes_and_broken_escapes_give_nothing() {
    assert_eq!(path_from_uri("https://example.com/a"), None);
    assert_eq!(path_from_uri("file:///a%2"), None);
    assert_eq!(path_from_uri("file:///a%zz"), None);
}
