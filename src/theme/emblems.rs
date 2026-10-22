use std::hash::{BuildHasher, RandomState};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
