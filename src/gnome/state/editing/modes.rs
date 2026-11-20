use std::path::PathBuf;

use trame::geometry::Point;
use trame::input::TextField;
use trame::ui::Hit;
use trame::ui::frame::{EditAnchor, EditTarget, SidebarItem};

use super::super::colors::Picker;
use super::super::settings::HostsPanel;

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
