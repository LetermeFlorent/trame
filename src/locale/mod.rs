mod detect;
mod strings;

use serde::{Deserialize, Serialize};

pub use detect::{language_of_id, language_of_tag, system_language};
pub use strings::Strings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    French,
}

impl Language {
    pub fn strings(self) -> &'static Strings {
        match self {
            Language::English => &strings::ENGLISH,
            Language::French => &strings::FRENCH,
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
    pub fn resolve(self) -> Language {
        match self {
            LanguageChoice::Auto => system_language(),
            LanguageChoice::French => Language::French,
            LanguageChoice::English => Language::English,
        }
    }
}
