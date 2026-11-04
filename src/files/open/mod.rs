mod launch;

pub use launch::open_link;

pub fn is_web(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    let known = ["http", "https"]
        .iter()
        .any(|allowed| scheme.eq_ignore_ascii_case(allowed));
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let clean = !url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '"' | '<' | '>' | '`'));
    known && clean && !host.is_empty()
}
