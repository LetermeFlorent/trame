mod look;

use std::path::PathBuf;

use crate::preferences::SshHost;
use crate::theme::{Emblem, Swatch};
use crate::workspace::EditTarget;

use super::{Desk, Effects};

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    Name { text: String, original: String },
    Color(Option<Swatch>),
    Fill(bool),
    Emblem(Emblem),
    CustomColor { slot: Option<usize>, color: Swatch },
    Folder(PathBuf),
    Host(SshHost),
}

impl Desk {
    pub fn commit(&mut self, target: EditTarget, edit: Edit) -> Effects {
        match edit {
            Edit::Name { text, original } => self.rename(target, &text, &original),
            Edit::Color(color) => self.paint(target, color),
            Edit::Fill(on) => self.fill(target, on),
            Edit::Emblem(emblem) => self.set_emblem(target, emblem),
            Edit::CustomColor { slot, color } => return self.custom_color(target, slot, color),
            Edit::Folder(folder) => return self.folder_picked(target.into(), folder),
            Edit::Host(host) => return self.connect(target, host),
        }
        Effects::default()
    }

    fn rename(&mut self, target: EditTarget, text: &str, original: &str) {
        if text == original {
            return;
        }
        match target {
            EditTarget::Tab(id) => self.workspace.rename_tab(id, text),
            EditTarget::Instance(id) => self.workspace.rename_instance(id, text),
        }
    }

    fn paint(&mut self, target: EditTarget, color: Option<Swatch>) {
        match target {
            EditTarget::Tab(id) => self.workspace.set_tab_color(id, color),
            EditTarget::Instance(id) => self.workspace.set_instance_color(id, color),
        }
    }

    fn fill(&mut self, target: EditTarget, on: bool) {
        match target {
            EditTarget::Tab(id) => self.workspace.set_tab_fill(id, on),
            EditTarget::Instance(id) => self.workspace.set_instance_fill(id, on),
        }
    }

    fn set_emblem(&mut self, target: EditTarget, emblem: Emblem) {
        if let EditTarget::Instance(id) = target {
            self.workspace.set_instance_emblem(id, emblem);
        }
    }

    fn custom_color(&mut self, target: EditTarget, slot: Option<usize>, color: Swatch) -> Effects {
        let custom = &self.preferences.custom_colors;
        let old = slot.and_then(|slot| custom.get(slot).copied());
        let next = self.preferences.with_custom_color(slot, color);
        let effects = self.store_preferences(next);
        if let Some(old) = old.filter(|old| *old != color) {
            self.workspace.replace_color(old, color);
        }
        self.paint(target, Some(color));
        effects
    }
}
