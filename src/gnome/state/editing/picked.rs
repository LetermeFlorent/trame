use super::{EditMode, Editing};
use crate::gnome::state::WindowState;
use trame::ui::frame::EditTarget;

impl WindowState {
    pub fn folder_picked(&mut self, target: trame::desk::FolderTarget, folder: std::path::PathBuf) {
        if let (
            trame::desk::FolderTarget::Instance(id),
            Some(Editing {
                target: EditTarget::Instance(eid),
                mode: EditMode::Folder { chosen, .. },
                ..
            }),
        ) = (target, &mut self.editing)
            && *eid == id
        {
            *chosen = Some(folder);
            return self.changed();
        }
        let effects = self.desk.folder_picked(target, folder);
        self.apply(effects);
    }

    pub fn icon_picked(&mut self, id: trame::workspace::InstanceId, image: u64) {
        let target = EditTarget::Instance(id);
        if self.editing.as_ref().is_some_and(|e| e.target == target) {
            self.editing = None;
        }
        let effects = self.desk.icon_picked(id, image);
        self.apply(effects);
    }
}
