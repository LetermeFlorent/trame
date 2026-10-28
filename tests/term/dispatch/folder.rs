use crate::feed;

const LOCAL: &str = "\x1b]9;9;C:\\work\x07";

#[test]
fn osc_7_from_another_host_is_ignored() {
    let term = feed(1, 10, "\x1b]7;file://elsewhere.example/home/me\x07");
    assert_eq!(term.screen().cwd(), None);
    let switch = format!("{LOCAL}\x1b]7;file://elsewhere/C:/Users\x07");
    let term = feed(1, 10, &switch);
    assert_eq!(term.screen().cwd(), Some("C:\\work"));
}

#[test]
fn osc_7_from_this_machine_is_kept() {
    let cases = [
        ("file:///home/me", "/home/me"),
        ("file://localhost/home/me", "/home/me"),
        ("file://LOCALHOST/C:/work", "C:/work"),
        ("file:///c:/Users/Me%20Doc", "c:/Users/Me Doc"),
    ];
    for (url, folder) in cases {
        let term = feed(1, 10, &format!("\x1b]7;{url}\x07"));
        assert_eq!(term.screen().cwd(), Some(folder), "{url}");
    }
}

#[cfg(windows)]
#[test]
fn osc_7_from_the_computer_name_is_kept() {
    let name = std::env::var("COMPUTERNAME").expect("a computer name");
    let url = format!("\x1b]7;file://{}/C:/work\x07", name.to_lowercase());
    let term = feed(1, 10, &url);
    assert_eq!(term.screen().cwd(), Some("C:/work"));
}

#[test]
fn osc_9_9_with_a_network_path_is_ignored() {
    let cases = [
        "\"\\\\server\\share\"",
        "\\\\server\\share",
        "//server/share",
        "/\\server\\share",
        "\\\\?\\UNC\\server\\share",
        "\\\\.\\pipe\\name",
    ];
    for path in cases {
        let term = feed(1, 10, &format!("{LOCAL}\x1b]9;9;{path}\x07"));
        assert_eq!(term.screen().cwd(), Some("C:\\work"), "{path}");
    }
}

#[test]
fn osc_9_9_with_a_relative_path_is_ignored() {
    for path in ["\\work", "work", "C:work", ""] {
        let term = feed(1, 10, &format!("{LOCAL}\x1b]9;9;{path}\x07"));
        assert_eq!(term.screen().cwd(), Some("C:\\work"), "{path}");
    }
}

#[test]
fn osc_7_with_a_network_path_or_a_control_character_is_ignored() {
    let cases = [
        "file:////server/share",
        "file://localhost//server/share",
        "file:///%5C%5Cserver%5Cshare",
        "file:///C:/work%0A",
        "file:///C:/work%1B",
    ];
    for url in cases {
        let term = feed(1, 10, &format!("{LOCAL}\x1b]7;{url}\x07"));
        assert_eq!(term.screen().cwd(), Some("C:\\work"), "{url}");
    }
}
