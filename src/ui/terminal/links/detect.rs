const SECURE_SCHEME: &str = "https://";
const WWW_PREFIX: &str = "www.";
const WEB_STARTS: [&str; 3] = [SECURE_SCHEME, "http://", WWW_PREFIX];
const URL_STOPS: &[char] = &['<', '>', '"', '`', '{', '}', '|', '\\', '^'];
const TRAILING: &[char] = &['.', ',', ';', ':', '!', '?', '\'', '"'];
const PAIRS: [(char, char); 4] = [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub start: usize,
    pub end: usize,
    pub target: String,
}

pub fn find_links(text: &[char]) -> Vec<Found> {
    let mut found = Vec::new();
    let mut start = 0;
    while start < text.len() {
        if text[start].is_whitespace() {
            start += 1;
            continue;
        }
        let end = text[start..]
            .iter()
            .position(|c| c.is_whitespace())
            .map_or(text.len(), |offset| start + offset);
        found.extend(web_link(text, start, end));
        start = end;
    }
    found
}

fn web_link(text: &[char], start: usize, end: usize) -> Option<Found> {
    let (from, kind) = (start..end).find_map(|at| {
        let word_start = at == start || !text[at - 1].is_alphanumeric();
        let kind = WEB_STARTS
            .iter()
            .find(|prefix| word_start && starts_with(&text[at..end], prefix))?;
        Some((at, kind))
    })?;
    let stop = text[from..end]
        .iter()
        .position(|c| URL_STOPS.contains(c))
        .map_or(end, |offset| from + offset);
    let stop = trimmed(text, from, stop);
    let body = &text[from + kind.len()..stop.max(from + kind.len())];
    if !body.first().is_some_and(|c| c.is_alphanumeric()) {
        return None;
    }
    let link: String = text[from..stop].iter().collect();
    let url = match *kind {
        WWW_PREFIX => format!("{SECURE_SCHEME}{link}"),
        _ => link,
    };
    Some(Found {
        start: from,
        end: stop,
        target: url,
    })
}

fn trimmed(text: &[char], start: usize, mut end: usize) -> usize {
    while end > start {
        let last = text[end - 1];
        let unmatched = PAIRS
            .iter()
            .find(|(_, close)| *close == last)
            .is_some_and(|(open, _)| {
                let inside = &text[start..end];
                let opens = inside.iter().filter(|c| *c == open).count();
                opens < inside.iter().filter(|c| **c == last).count()
            });
        if !TRAILING.contains(&last) && !unmatched {
            break;
        }
        end -= 1;
    }
    end
}

fn starts_with(text: &[char], prefix: &str) -> bool {
    text.len() >= prefix.len()
        && prefix
            .chars()
            .zip(text)
            .all(|(wanted, seen)| seen.eq_ignore_ascii_case(&wanted))
}
