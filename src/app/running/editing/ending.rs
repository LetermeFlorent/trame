use trame::desk::Edit;
use trame::theme::{Emblem, Letters, Swatch};

use super::EditMode;

pub(super) enum Ending {
    Commit(Edit),
    Keep,
    Close,
}

impl EditMode {
    pub(super) fn ending(&mut self) -> Ending {
        let edit = match self {
            EditMode::Text { field, original } => Edit::Name {
                text: field.text().to_owned(),
                original: original.clone(),
            },
            EditMode::Folder { original, chosen } => {
                match chosen
                    .take()
                    .filter(|folder| Some(folder) != original.as_ref())
                {
                    Some(folder) => Edit::Folder(folder),
                    None => return Ending::Close,
                }
            }
            EditMode::Picker(picker) => Edit::CustomColor {
                slot: picker.slot,
                color: Swatch::custom(picker.hsv.to_rgb()),
            },
            EditMode::Emblem { letters } => match Letters::parse(letters.text()) {
                Some(letters) => Edit::Emblem(Emblem::Letters(letters)),
                None => return Ending::Keep,
            },
            EditMode::Form(_) | EditMode::Color { .. } => return Ending::Close,
        };
        Ending::Commit(edit)
    }
}
