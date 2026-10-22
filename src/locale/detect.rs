use super::Language;

const FRENCH_TAG: &str = "fr";
const TAG_SEPARATORS: [char; 4] = ['_', '-', '.', '@'];
const PRIMARY_LANGUAGE_MASK: u16 = 0x3ff;
const PRIMARY_FRENCH: u16 = 0x0c;

#[cfg(not(windows))]
const LOCALE_VARIABLES: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

pub fn language_of_tag(tag: &str) -> Language {
    let primary = tag.trim().split(TAG_SEPARATORS).next().unwrap_or_default();
    match primary.eq_ignore_ascii_case(FRENCH_TAG) {
        true => Language::French,
        false => Language::English,
    }
}

pub fn language_of_id(id: u16) -> Language {
    match id & PRIMARY_LANGUAGE_MASK == PRIMARY_FRENCH {
        true => Language::French,
        false => Language::English,
    }
}

#[cfg(windows)]
pub fn system_language() -> Language {
    use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
    language_of_id(unsafe { GetUserDefaultUILanguage() })
}

#[cfg(not(windows))]
pub fn system_language() -> Language {
    LOCALE_VARIABLES
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty())
        .map_or(Language::English, |tag| language_of_tag(&tag))
}
