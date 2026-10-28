mod detect;
mod english;
mod french;
mod keys;
mod strings;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use detect::{language_of_id, language_of_tag, system_language};
pub use keys::KeyStrings;
pub use strings::{ALIAS_SLOT, NUMBER_SLOT, Strings, VERSION_SLOT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    French,
}

impl Language {
    pub fn strings(self) -> &'static Strings {
        match self {
            Language::English => &english::ENGLISH,
            Language::French => &french::FRENCH,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageChoice {
    #[default]
    Auto,
    French,
    English,
}

impl LanguageChoice {
    pub const ALL: [Self; 3] = [Self::Auto, Self::French, Self::English];

    pub fn label(self) -> &'static str {
        match self {
            LanguageChoice::Auto => "Auto",
            LanguageChoice::French => "Français",
            LanguageChoice::English => "English",
        }
    }

    pub fn resolve(self) -> Language {
        match self {
            LanguageChoice::Auto => system_language(),
            LanguageChoice::French => Language::French,
            LanguageChoice::English => Language::English,
        }
    }
}
