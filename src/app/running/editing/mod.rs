mod commit;
mod folder;
mod keys;
mod menu;
mod pointer;

use std::path::PathBuf;

use trame::input::TextField;
use trame::theme::{Emblem, Swatch};
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditKind, EditTarget, EditorRequest, MenuEntry, SidebarItem};

use super::Running;
use super::settings::HostsPanel;
use folder::shown_folder;
pub(super) use menu::Menu;

enum EditMode {
    Text {
        field: TextField,
        original: String,
    },
    Form(HostsPanel),
    Color,
    Emblem,
    Folder {
        original: Option<PathBuf>,
        chosen: Option<PathBuf>,
    },
}

pub(super) struct Editing {
    target: EditTarget,
    anchor: EditAnchor,
    mode: EditMode,
}

impl Editing {
    pub(super) fn panel_mut(&mut self) -> Option<&mut HostsPanel> {
        match &mut self.mode {
            EditMode::Form(panel) => Some(panel),
            _ => None,
        }
    }
}

fn target_of(hit: Hit) -> Option<(EditTarget, EditAnchor)> {
    match hit {
        Hit::Tab(pane, id) => Some((EditTarget::Tab(id), EditAnchor::Strip(pane))),
        Hit::Row(SidebarItem::Tab(id)) => Some((EditTarget::Tab(id), EditAnchor::Sidebar)),
        Hit::Row(SidebarItem::Instance(id)) => {
            Some((EditTarget::Instance(id), EditAnchor::Sidebar))
        }
        Hit::RailInstance(id) => Some((EditTarget::Instance(id), EditAnchor::Rail)),
        _ => None,
    }
}

impl Running {
    pub(super) fn open_editor(&mut self, hit: Hit) -> bool {
        let Some((target, anchor)) = target_of(hit) else {
            return false;
        };
        self.start_edit(target, anchor, MenuEntry::Rename);
        true
    }

    fn start_edit(&mut self, target: EditTarget, anchor: EditAnchor, entry: MenuEntry) {
        let mode = match entry {
            MenuEntry::Rename => {
                let original = self.name_of(target);
                let mut field = TextField::new(&original);
                field.select_all();
                EditMode::Text { field, original }
            }
            MenuEntry::Color => EditMode::Color,
            MenuEntry::Icon => EditMode::Emblem,
            MenuEntry::Folder => self.folder_mode(target),
            MenuEntry::Ssh => EditMode::Form(HostsPanel::connecting(self.known_hosts())),
        };
        self.editing = Some(Editing {
            target,
            anchor,
            mode,
        });
        self.changed();
    }

    pub(super) fn place_editor(&mut self) {
        let Some(editing) = &self.editing else {
            return;
        };
        let kind = match &editing.mode {
            EditMode::Folder { chosen, .. } => {
                return self.place_folder(shown_folder(chosen.as_ref()));
            }
            EditMode::Text { field, .. } => EditKind::Name(field),
            EditMode::Form(panel) => EditKind::Connect(panel.view()),
            EditMode::Color => EditKind::Color(self.color_of(editing.target)),
            EditMode::Emblem => EditKind::Emblem(self.emblem_of(editing.target)),
        };
        let request = EditorRequest {
            target: editing.target,
            anchor: editing.anchor,
            kind,
            strings: self.strings,
        };
        self.frame.place_editor(&mut self.text, &request);
        if self.frame.editor.is_none() {
            self.editing = None;
        }
    }

    fn name_of(&self, target: EditTarget) -> String {
        match target {
            EditTarget::Tab(id) => self.tab_title(id),
            EditTarget::Instance(id) => self
                .workspace
                .instance(id)
                .map(|instance| instance.name.clone())
                .unwrap_or_default(),
        }
    }

    fn color_of(&self, target: EditTarget) -> Option<Swatch> {
        match target {
            EditTarget::Tab(id) => self.workspace.tab_color(id),
            EditTarget::Instance(id) => self.workspace.instance(id)?.color,
        }
    }

    fn emblem_of(&self, target: EditTarget) -> Option<Emblem> {
        match target {
            EditTarget::Tab(_) => None,
            EditTarget::Instance(id) => Some(self.workspace.instance(id)?.emblem),
        }
    }
}
