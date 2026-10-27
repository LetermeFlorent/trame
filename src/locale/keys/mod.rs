mod english;
mod french;

pub use english::ENGLISH_KEYS;
pub use french::FRENCH_KEYS;

#[derive(Debug, PartialEq, Eq)]
pub struct KeyStrings {
    pub tab: &'static str,
    pub all_default: &'static str,
    pub default: &'static str,
    pub back: &'static str,
    pub add_key: &'static str,
    pub press_keys: &'static str,
    pub no_key: &'static str,
    pub taken_by: &'static str,
    pub typing: &'static str,
    pub unsupported: &'static str,
    pub shift: &'static str,
    pub actions: [&'static str; 32],
    pub named: [&'static str; 15],
}
