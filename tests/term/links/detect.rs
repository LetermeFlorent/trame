use trame::files::LinkTarget;
use trame::ui::terminal::links::find_links;

fn links(text: &str) -> Vec<(String, LinkTarget)> {
    let chars: Vec<char> = text.chars().collect();
    find_links(&chars)
        .into_iter()
        .map(|found| (chars[found.start..found.end].iter().collect(), found.target))
        .collect()
}

fn only(text: &str) -> (String, LinkTarget) {
    let found = links(text);
    assert_eq!(found.len(), 1, "{text}: {found:?}");
    found[0].clone()
}

fn web(url: &str) -> LinkTarget {
    LinkTarget::Web(url.to_owned())
}

fn path(path: &str) -> LinkTarget {
    LinkTarget::Path(path.to_owned())
}

#[test]
fn web_addresses_are_found_with_their_scheme() {
    for url in [
        "https://example.com/a?b=1&c#d",
        "http://x.org",
        "HTTPS://Example.com",
    ] {
        let (shown, target) = only(&format!("see {url} now"));
        assert_eq!(shown, url);
        assert_eq!(target, web(url));
    }
    let (shown, target) = only("go to www.example.com/docs");
    assert_eq!(shown, "www.example.com/docs");
    assert_eq!(target, web("https://www.example.com/docs"));
}

#[test]
fn trailing_punctuation_stays_out_of_a_web_address() {
    let cases = [
        ("(see https://example.com/a).", "https://example.com/a"),
        ("https://example.com/a, then", "https://example.com/a"),
        ("'https://example.com/a'", "https://example.com/a"),
        ("\"https://example.com/a\":", "https://example.com/a"),
        ("https://example.com/a!?;", "https://example.com/a"),
        ("[https://example.com/a]", "https://example.com/a"),
        (
            "https://en.wikipedia.org/wiki/Rust_(language).",
            "https://en.wikipedia.org/wiki/Rust_(language)",
        ),
        ("<https://example.com/a>", "https://example.com/a"),
    ];
    for (text, url) in cases {
        assert_eq!(only(text), (url.to_owned(), web(url)), "{text}");
    }
}

#[test]
fn every_kind_of_path_is_found() {
    for text in [
        r"C:\dir\file.txt",
        "C:/dir/file.txt",
        r"\\server\share\file.txt",
        "~/notes.md",
        "./run",
        "../up/file",
        "/usr/bin/env",
        "src/main.rs",
        r"src\ui\mod.rs",
    ] {
        let found = only(&format!("open {text} now"));
        assert_eq!(found, (text.to_owned(), path(text)));
    }
    assert_eq!(only("(src/main.rs)").1, path("src/main.rs"));
}

#[test]
fn a_line_and_column_suffix_is_underlined_but_not_opened() {
    let cases = [
        ("src/main.rs:12:5", "src/main.rs:12:5", "src/main.rs"),
        ("src/main.rs:12:", "src/main.rs:12", "src/main.rs"),
        ("/usr/lib/x.so:3", "/usr/lib/x.so:3", "/usr/lib/x.so"),
        (
            r"C:\dir\file.txt:7",
            r"C:\dir\file.txt:7",
            r"C:\dir\file.txt",
        ),
    ];
    for (text, shown, opened) in cases {
        assert_eq!(only(text), (shown.to_owned(), path(opened)), "{text}");
    }
}

#[test]
fn plain_words_dates_and_bare_slashes_are_not_links() {
    for text in [
        "2026/11/04",
        "/",
        "/usr",
        "/usr/",
        "and/or",
        "hello.txt",
        "km/h.",
        "1/2",
        "a//b.rs",
        "foo=src/main.rs",
        r"C:\",
        r"\\server",
        "~/",
        "xhttps://example.com",
        "awww.example.com",
        "http://",
        "www.",
        "https://.",
        "mailto:me@example.com",
        "just some words.",
    ] {
        assert!(links(text).is_empty(), "{text}: {:?}", links(text));
    }
}

#[test]
fn several_links_on_one_line_are_all_found() {
    let found = links("a https://one.example b src/two.rs c /etc/three/file");
    let targets: Vec<LinkTarget> = found.into_iter().map(|(_, target)| target).collect();
    let wanted = [
        web("https://one.example"),
        path("src/two.rs"),
        path("/etc/three/file"),
    ];
    assert_eq!(targets, wanted);
}
