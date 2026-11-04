use trame::files::is_web;

#[test]
fn only_http_and_https_addresses_go_to_the_browser() {
    for url in ["https://example.com/a", "http://x.org", "HTTPS://A.EXAMPLE"] {
        assert!(is_web(url), "{url}");
    }
    for uri in [
        "javascript:alert(1)",
        "ftp://example.com/file",
        "ms-settings:display",
        "vscode://file/c:/x",
        "file://elsewhere.example/etc/hosts",
        "file:///C:/dir/file.txt",
        "https://",
        "https:///path",
        "https://a.example/\"x",
        "https://a.example/ x",
    ] {
        assert!(!is_web(uri), "{uri}");
    }
}
