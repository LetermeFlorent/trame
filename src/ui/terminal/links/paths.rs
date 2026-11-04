const PATH_MARKS: &[char] = &[
    '.', '_', '-', '~', '/', '\\', '@', '+', '%', '#', ',', '(', ')', '[', ']',
];
const MAX_EXTENSION: usize = 8;

pub(super) fn without_line(chars: &[char]) -> usize {
    let mut end = chars.len();
    for _ in 0..2 {
        let digits = chars[..end]
            .iter()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .count();
        if digits == 0 || digits >= end || chars[end - digits - 1] != ':' {
            break;
        }
        end -= digits + 1;
    }
    end
}

pub(super) fn is_path(chars: &[char]) -> bool {
    let text: String = chars.iter().collect();
    let drive = is_drive(chars);
    let allowed = chars
        .iter()
        .enumerate()
        .all(|(at, c)| c.is_alphanumeric() || PATH_MARKS.contains(c) || (drive && at == 1));
    allowed && (drive && chars.len() > 3 || is_anchored(&text) || is_relative(&text))
}

fn is_drive(chars: &[char]) -> bool {
    chars.len() >= 3 && chars[0].is_ascii_alphabetic() && chars[1] == ':' && is_slash(chars[2])
}

fn is_anchored(text: &str) -> bool {
    if let Some(share) = text.strip_prefix(r"\\") {
        return filled(share.split('\\').collect(), 2);
    }
    if let Some(rest) = ["~/", "~\\", "./", ".\\", "../", "..\\"]
        .iter()
        .find_map(|start| text.strip_prefix(start))
    {
        return !rest.is_empty() && !rest.starts_with(is_slash);
    }
    match text.strip_prefix('/') {
        Some(rest) => filled(rest.split('/').collect(), 2),
        None => false,
    }
}

fn is_relative(text: &str) -> bool {
    let parts: Vec<&str> = text.split(is_slash).collect();
    let Some(last) = parts.last() else {
        return false;
    };
    parts.len() >= 2
        && parts.iter().all(|part| !part.is_empty())
        && text.starts_with(|c: char| c.is_alphanumeric() || matches!(c, '_' | '.'))
        && has_extension(last)
}

fn has_extension(name: &str) -> bool {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && extension.len() <= MAX_EXTENSION
        && extension.chars().all(|c| c.is_ascii_alphanumeric())
        && extension.chars().any(|c| c.is_ascii_alphabetic())
}

fn filled(mut parts: Vec<&str>, least: usize) -> bool {
    if parts.len() > 1 && parts.last() == Some(&"") {
        parts.pop();
    }
    parts.len() >= least && parts.iter().all(|part| !part.is_empty())
}

fn is_slash(c: char) -> bool {
    matches!(c, '/' | '\\')
}
