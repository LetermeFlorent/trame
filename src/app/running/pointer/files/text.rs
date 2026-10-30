use std::path::Path;

const VERBATIM_UNC: &str = r"\\?\UNC\";
const VERBATIM: &str = r"\\?\";
const UNC: &str = r"\\";

pub(super) fn dropped_text<'a>(paths: impl IntoIterator<Item = &'a Path>) -> String {
    paths
        .into_iter()
        .filter_map(typed_path)
        .map(|path| format!("{path} "))
        .collect()
}

fn typed_path(path: &Path) -> Option<String> {
    let text = path.to_str()?;
    if text.is_empty() || text.chars().any(char::is_control) {
        return None;
    }
    let plain = match text.strip_prefix(VERBATIM_UNC) {
        Some(share) => format!("{UNC}{share}"),
        None => text.strip_prefix(VERBATIM).unwrap_or(text).to_owned(),
    };
    Some(plain)
}
