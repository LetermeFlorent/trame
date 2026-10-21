use super::Appearance;

const COLOR_SCHEME_KEY: &str = "color-scheme";
const GTK_THEME_KEY: &str = "gtk-theme";
const PREFER_DARK: &str = "prefer-dark";
const PREFER_LIGHT: &str = "prefer-light";
const DARK_THEME_MARKER: &str = "dark";

pub fn parse_color_scheme(output: &str) -> Option<Appearance> {
    match unquote(output).to_ascii_lowercase().as_str() {
        PREFER_DARK => Some(Appearance::Dark),
        PREFER_LIGHT => Some(Appearance::Light),
        _ => None,
    }
}

pub fn parse_gtk_theme(output: &str) -> Option<Appearance> {
    let name = unquote(output).to_ascii_lowercase();
    if name.is_empty() {
        return None;
    }
    if name.contains(DARK_THEME_MARKER) {
        return Some(Appearance::Dark);
    }
    Some(Appearance::Light)
}

pub fn detect_with(read: impl Fn(&str) -> Option<String>) -> Option<Appearance> {
    read(COLOR_SCHEME_KEY)
        .as_deref()
        .and_then(parse_color_scheme)
        .or_else(|| read(GTK_THEME_KEY).as_deref().and_then(parse_gtk_theme))
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn detect() -> Option<Appearance> {
    detect_with(read_setting)
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
pub fn detect() -> Option<Appearance> {
    None
}

fn unquote(output: &str) -> &str {
    let text = output.trim();
    let text = text.strip_prefix("@s ").unwrap_or(text);
    text.trim_matches('\'').trim()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn read_setting(key: &str) -> Option<String> {
    use std::process::{Command, Stdio};
    const SCHEMA: &str = "org.gnome.desktop.interface";
    let output = Command::new("gsettings")
        .args(["get", SCHEMA, key])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}
