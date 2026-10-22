#[derive(Debug, PartialEq, Eq)]
pub struct Strings {
    pub checking_updates: &'static str,
    pub downloading_version: &'static str,
}

pub const ENGLISH: Strings = Strings {
    checking_updates: "Checking for updates",
    downloading_version: "Downloading version",
};

pub const FRENCH: Strings = Strings {
    checking_updates: "Recherche de mises à jour",
    downloading_version: "Téléchargement de la version",
};
