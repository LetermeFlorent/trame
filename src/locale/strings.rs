#[derive(Debug, PartialEq, Eq)]
pub struct Strings {
    pub checking_updates: &'static str,
    pub downloading_version: &'static str,
    pub installing_version: &'static str,
    pub rename: &'static str,
    pub change_color: &'static str,
    pub change_icon: &'static str,
}

pub const ENGLISH: Strings = Strings {
    checking_updates: "Checking for updates",
    downloading_version: "Downloading version",
    installing_version: "Installing version",
    rename: "Rename",
    change_color: "Change colour",
    change_icon: "Change icon",
};

pub const FRENCH: Strings = Strings {
    checking_updates: "Recherche de mises à jour",
    downloading_version: "Téléchargement de la version",
    installing_version: "Installation de la version",
    rename: "Renommer",
    change_color: "Changer de couleur",
    change_icon: "Changer d'icône",
};
