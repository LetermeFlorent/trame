use std::path::{Component, Path, PathBuf};

use crate::files::pictures::has_image_extension;
use crate::files::{home_dir, is_web};

const FILE_SCHEME: &str = "file://";
const LOCALHOST: &str = "localhost";
const WEB_PREFIXES: [&str; 3] = ["https://", "http://", "www."];

fn starts_ignore_ascii(raw: &str, prefix: &str) -> bool {
    raw.get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

pub(super) fn resolve_candidate(
    raw: &str,
    cwd: Option<&Path>,
    is_file: impl FnOnce(&Path) -> bool,
) -> Option<PathBuf> {
    if WEB_PREFIXES.iter().any(|p| starts_ignore_ascii(raw, p)) {
        return None;
    }
    resolve_image_with(raw, cwd, is_file)
}

pub(super) fn overlaps(claimed: &[(usize, usize)], start: usize, end: usize) -> bool {
    claimed.iter().any(|&(s, e)| start < e && s < end)
}

pub(super) fn starts_chars(text: &[char], prefix: &str) -> bool {
    text.len() >= prefix.len()
        && prefix
            .chars()
            .zip(text)
            .all(|(want, got)| got.eq_ignore_ascii_case(&want))
}

pub(super) fn is_unc(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::path::Prefix;
        if let Some(Component::Prefix(prefix)) = path.components().next() {
            return matches!(prefix.kind(), Prefix::UNC(..) | Prefix::VerbatimUNC(..));
        }
    }
    path.to_string_lossy().starts_with(r"\\")
}

pub(super) fn check_image_file(path: &Path, is_file: impl FnOnce(&Path) -> bool) -> bool {
    !is_unc(path) && has_image_extension(path) && is_file(path)
}

pub(super) fn resolve_cell_link(
    url: &str,
    is_file: impl FnOnce(&Path) -> bool,
) -> Option<(super::LinkKind, String)> {
    if is_web(url) {
        Some((super::LinkKind::Web, url.to_owned()))
    } else {
        let path = file_uri_path(url).filter(|p| check_image_file(p, is_file))?;
        Some((super::LinkKind::Image, path.to_string_lossy().into_owned()))
    }
}

pub fn resolve_image_path(raw: &str, cwd: Option<&Path>) -> Option<PathBuf> {
    resolve_image_with(raw, cwd, Path::is_file)
}

fn home_subpath(tail: &str) -> Option<&Path> {
    let tail = tail.trim_start_matches(['/', '\\']);
    let sub = Path::new(tail);
    let invalid = tail.is_empty()
        || sub.has_root()
        || matches!(
            sub.components().next(),
            Some(Component::Prefix(_) | Component::RootDir)
        );
    (!invalid).then_some(sub)
}

pub(super) fn resolve_image_with(
    raw: &str,
    cwd: Option<&Path>,
    is_file: impl FnOnce(&Path) -> bool,
) -> Option<PathBuf> {
    if raw.is_empty() || raw.chars().any(char::is_control) {
        return None;
    }
    if starts_ignore_ascii(raw, FILE_SCHEME) {
        return file_uri_path(raw).filter(|p| check_image_file(p, is_file));
    }
    let candidate = if let Some(tail) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
        home_dir()?.join(home_subpath(tail)?)
    } else {
        let path = Path::new(raw);
        if path.is_absolute() {
            path.to_path_buf()
        } else if matches!(path.components().next(), Some(Component::Prefix(_))) {
            return None;
        } else {
            cwd?.join(path)
        }
    };
    check_image_file(&candidate, is_file).then_some(candidate)
}

pub(super) fn file_uri_path(raw: &str) -> Option<PathBuf> {
    if !starts_ignore_ascii(raw, FILE_SCHEME) {
        return None;
    }
    let rest = &raw[FILE_SCHEME.len()..];
    let slash = rest.find('/')?;
    let (host, tail) = rest.split_at(slash);
    if !host.is_empty() && !host.eq_ignore_ascii_case(LOCALHOST) {
        return None;
    }
    let decoded = percent_decode(tail.as_bytes())?;
    let trimmed = match decoded.as_bytes() {
        [b'/', drive, b':', ..] if cfg!(windows) && drive.is_ascii_alphabetic() => &decoded[1..],
        _ => &decoded,
    };
    Some(PathBuf::from(trimmed))
}

fn percent_decode(bytes: &[u8]) -> Option<String> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = bytes
                .get(i + 1..i + 3)
                .and_then(|s| std::str::from_utf8(s).ok())
                .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))
            && let Ok(val) = u8::from_str_radix(hex, 16)
        {
            out.push(val);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).ok()
}
