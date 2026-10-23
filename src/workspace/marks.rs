use std::path::PathBuf;

use super::{Instance, InstanceId, Workspace};
use crate::preferences::SshHost;
use crate::session::SessionId;
use crate::theme::{Emblem, Swatch};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TabMark {
    pub name: Option<String>,
    pub color: Option<Swatch>,
    pub host: Option<SshHost>,
}

impl TabMark {
    fn is_blank(&self) -> bool {
        self.name.is_none() && self.color.is_none() && self.host.is_none()
    }
}

pub fn clean_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

impl Workspace {
    pub fn tab_name(&self, tab: SessionId) -> Option<&str> {
        self.marks.get(&tab)?.name.as_deref()
    }

    pub fn tab_color(&self, tab: SessionId) -> Option<Swatch> {
        self.marks.get(&tab)?.color
    }

    pub fn tab_host(&self, tab: SessionId) -> Option<&SshHost> {
        self.marks.get(&tab)?.host.as_ref()
    }

    pub fn set_tab_host(&mut self, tab: SessionId, host: Option<SshHost>) {
        self.edit_mark(tab, |mark| mark.host = host);
    }

    pub fn rename_tab(&mut self, tab: SessionId, name: &str) {
        let name = clean_name(name);
        self.edit_mark(tab, |mark| mark.name = name);
    }

    pub fn set_tab_color(&mut self, tab: SessionId, color: Option<Swatch>) {
        self.edit_mark(tab, |mark| mark.color = color);
    }

    pub fn rename_instance(&mut self, id: InstanceId, name: &str) {
        if let Some(instance) = self.instance_mut(id) {
            instance.name = clean_name(name).unwrap_or_else(|| Instance::automatic_name(id));
        }
    }

    pub fn set_instance_color(&mut self, id: InstanceId, color: Option<Swatch>) {
        if let Some(instance) = self.instance_mut(id) {
            instance.color = color;
        }
    }

    pub fn set_instance_emblem(&mut self, id: InstanceId, emblem: Emblem) {
        if let Some(instance) = self.instance_mut(id) {
            instance.emblem = emblem;
        }
    }

    pub fn set_instance_folder(&mut self, id: InstanceId, folder: Option<PathBuf>) {
        if let Some(instance) = self.instance_mut(id) {
            instance.folder = folder;
        }
    }

    pub(super) fn restore_mark(&mut self, tab: SessionId, mark: TabMark) {
        if !mark.is_blank() {
            self.marks.insert(tab, mark);
        }
    }

    pub(super) fn forget_marks(&mut self, tabs: &[SessionId]) {
        for tab in tabs {
            self.marks.remove(tab);
        }
    }

    pub fn instance(&self, id: InstanceId) -> Option<&Instance> {
        self.instances.iter().find(|instance| instance.id == id)
    }

    fn instance_mut(&mut self, id: InstanceId) -> Option<&mut Instance> {
        self.instances.iter_mut().find(|instance| instance.id == id)
    }

    fn edit_mark(&mut self, tab: SessionId, change: impl FnOnce(&mut TabMark)) {
        if self.locate(tab).is_none() {
            return;
        }
        let mark = self.marks.entry(tab).or_default();
        change(mark);
        if mark.is_blank() {
            self.marks.remove(&tab);
        }
    }
}
