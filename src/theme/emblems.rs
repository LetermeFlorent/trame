use std::hash::{BuildHasher, RandomState};

use serde::{Deserialize, Serialize};

use super::letters::Letters;

const LETTERS_PREFIX: &str = "letters:";
const IMAGE_PREFIX: &str = "image:";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Emblem {
    #[default]
    Grid,
    Terminal,
    Folder,
    Code,
    Server,
    Globe,
    Database,
    Gear,
    Letters(Letters),
    Image(u64),
}

impl Emblem {
    pub const ALL: [Emblem; 8] = [
        Emblem::Grid,
        Emblem::Terminal,
        Emblem::Folder,
        Emblem::Code,
        Emblem::Server,
        Emblem::Globe,
        Emblem::Database,
        Emblem::Gear,
    ];

    const NAMES: [&'static str; 8] = [
        "grid", "terminal", "folder", "code", "server", "globe", "database", "gear",
    ];

    pub fn parse(text: &str) -> Option<Emblem> {
        if let Some(index) = Self::NAMES.iter().position(|name| *name == text) {
            return Some(Self::ALL[index]);
        }
        if let Some(letters) = text.strip_prefix(LETTERS_PREFIX) {
            return Letters::parse(letters).map(Emblem::Letters);
        }
        let id = text.strip_prefix(IMAGE_PREFIX)?;
        u64::from_str_radix(id, 16).ok().map(Emblem::Image)
    }

    pub fn pick(taken: &[Emblem], seed: u64) -> Emblem {
        let free: Vec<Emblem> = Self::ALL
            .into_iter()
            .filter(|emblem| !taken.contains(emblem))
            .collect();
        let pool = match free.is_empty() {
            true => Self::ALL.to_vec(),
            false => free,
        };
        pool[(seed % pool.len() as u64) as usize]
    }

    pub fn random(taken: &[Emblem]) -> Emblem {
        Self::pick(taken, RandomState::new().hash_one(()))
    }
}

impl From<Emblem> for String {
    fn from(emblem: Emblem) -> String {
        match emblem {
            Emblem::Letters(letters) => format!("{LETTERS_PREFIX}{letters}"),
            Emblem::Image(id) => format!("{IMAGE_PREFIX}{id:016x}"),
            preset => {
                let index = Emblem::ALL.iter().position(|known| *known == preset);
                Emblem::NAMES[index.unwrap_or_default()].to_owned()
            }
        }
    }
}

impl TryFrom<String> for Emblem {
    type Error = String;

    fn try_from(text: String) -> Result<Emblem, String> {
        Emblem::parse(&text).ok_or_else(|| format!("unknown icon {text}"))
    }
}
