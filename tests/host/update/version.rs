use trame::host::update::RELEASES;
use trame::host::update::version::Version;

use crate::update::version;

#[test]
fn a_tag_reads_with_or_without_its_v() {
    assert_eq!(version("v1.2.3"), version("1.2.3"));
    assert_eq!(version("v0.10.7").to_string(), "v0.10.7");
    assert_eq!(version("v0.10.7").number(), "0.10.7");
}

#[test]
fn anything_but_three_plain_numbers_is_refused() {
    for text in [
        "",
        "v",
        "v1.2",
        "v1.2.3.4",
        "v1.2.3-rc1",
        "v1..3",
        "va.b.c",
        "v1.2.+3",
        "vv1.2.3",
    ] {
        assert_eq!(Version::parse(text), None, "{text}");
    }
}

#[test]
fn numbers_compare_as_numbers_not_as_text() {
    assert!(version("0.10.0") > version("0.9.9"));
    assert!(version("1.0.0") > version("0.99.99"));
    assert!(version("0.2.1") > version("0.2.0"));
    assert_eq!(
        version("0.2.0").cmp(&version("v0.2.0")),
        std::cmp::Ordering::Equal
    );
}

#[test]
fn the_running_version_is_readable() {
    assert!(Version::current().is_some());
}

#[test]
fn the_latest_release_comes_from_the_redirect() {
    let location = format!("{RELEASES}/tag/v0.3.1\n");
    assert_eq!(Version::released(&location), Some(version("0.3.1")));
}

#[test]
fn a_repository_without_release_or_another_page_gives_nothing() {
    for location in [
        String::new(),
        RELEASES.to_owned(),
        format!("{RELEASES}/tag/0.3.1"),
        format!("{RELEASES}/tag/v0.3.1-beta"),
        "https://github.com/someone/else/releases/tag/v9.0.0".to_owned(),
        "https://github.com/login".to_owned(),
    ] {
        assert_eq!(Version::released(&location), None, "{location}");
    }
}
