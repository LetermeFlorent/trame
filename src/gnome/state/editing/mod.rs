mod commit;
mod place;

use std::path::PathBuf;

use trame::geometry::Point;
use trame::input::TextField;
use trame::locale::Strings;
use trame::theme::Emblem;
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditTarget, MenuEntry, SidebarItem};

use super::WindowState;
use super::colors::Picker;
use super::settings::HostsPanel;

pub enum EditMode {
    Text {
        field: TextField,
        original: String,
    },
    Form(HostsPanel),
    Color {
        first: usize,
    },
    Picker(Picker),
    Emblem {
        letters: TextField,
    },
    Folder {
        original: Option<PathBuf>,
        chosen: Option<PathBuf>,
    },
}

pub struct Editing {
    pub target: EditTarget,
    pub anchor: EditAnchor,
    pub mode: EditMode,
}

pub struct Menu {
    pub target: EditTarget,
    pub anchor: EditAnchor,
    pub at: Point,
    pub adding: bool,
}

impl Editing {
    pub fn panel_mut(&mut self) -> Option<&mut HostsPanel> {
        match &mut self.mode {
            EditMode::Form(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn picker_mut(&mut self) -> Option<&mut Picker> {
        match &mut self.mode {
            EditMode::Picker(picker) => Some(picker),
            _ => None,
        }
    }

    pub fn letters_mut(&mut self) -> Option<&mut TextField> {
        match &mut self.mode {
            EditMode::Emblem { letters } => Some(letters),
            _ => None,
        }
    }
}

pub fn target_of(hit: Hit) -> Option<(EditTarget, EditAnchor)> {
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

impl WindowState {
    pub fn open_editor(&mut self, hit: Hit) -> bool {
        let Some((target, anchor)) = target_of(hit) else {
            return false;
        };
        self.start_edit(target, anchor, MenuEntry::Rename);
        true
    }

    pub fn start_edit(&mut self, target: EditTarget, anchor: EditAnchor, entry: MenuEntry) {
        let mode = match entry {
            MenuEntry::Rename => {
                let original = self.name_of(target);
                let mut field = TextField::new(&original);
                field.select_all();
                EditMode::Text { field, original }
            }
            MenuEntry::Color => EditMode::Color { first: 0 },
            MenuEntry::Icon => {
                let shown = match self.desk.emblem_of(target) {
                    Some(Emblem::Letters(l)) => l.to_string(),
                    _ => String::new(),
                };
                let mut letters = TextField::new(&shown);
                letters.select_all();
                EditMode::Emblem { letters }
            }
            MenuEntry::Folder => {
                let original = self.desk.folder_of(target.into());
                EditMode::Folder {
                    chosen: original.clone(),
                    original,
                }
            }
            MenuEntry::Ssh => EditMode::Form(HostsPanel::connecting(self.desk.known_hosts())),
            MenuEntry::NewTab => return,
        };
        self.editing = Some(Editing {
            target,
            anchor,
            mode,
        });
        self.changed();
    }

    fn name_of(&self, target: EditTarget) -> String {
        match target {
            EditTarget::Tab(id) => self.tab_title(id),
            EditTarget::Instance(id) => self
                .desk
                .workspace
                .instance(id)
                .map(|i| i.shown_name(self.desk.strings.instance_name).into_owned())
                .unwrap_or_default(),
        }
    }
}

pub(super) fn edit_entries(
    target: EditTarget,
    strings: &'static Strings,
) -> Vec<(MenuEntry, &'static str)> {
    let mut entries = vec![
        (MenuEntry::Rename, strings.rename),
        (MenuEntry::Color, strings.change_color),
    ];
    if let EditTarget::Instance(_) = target {
        entries.push((MenuEntry::Icon, strings.change_icon));
    }
    entries.push((MenuEntry::Folder, strings.change_folder));
    entries.push((MenuEntry::Ssh, strings.ssh_connection));
    entries
}
