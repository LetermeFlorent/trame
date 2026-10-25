use crate::geometry::Point;
use crate::locale::Strings;
use crate::preferences::Preferences;
use crate::workspace::Numbering;

use super::column::Column;
use super::{SettingControl, SettingLine, ToggleRow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberChoice {
    Shown,
    OnClose,
    OnMove,
}

impl NumberChoice {
    pub const ALL: [NumberChoice; 3] = [Self::Shown, Self::OnClose, Self::OnMove];

    pub fn toggle(self, numbering: &mut Numbering) {
        let flag = match self {
            Self::Shown => &mut numbering.shown,
            Self::OnClose => &mut numbering.on_close,
            Self::OnMove => &mut numbering.on_move,
        };
        *flag = !*flag;
    }

    pub fn state(self, numbering: Numbering) -> [bool; 2] {
        match self {
            Self::Shown => [numbering.shown, true],
            Self::OnClose => [numbering.on_close, numbering.shown],
            Self::OnMove => [numbering.on_move, numbering.shown],
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstancesPage {
    pub section: SettingLine,
    pub toggles: [ToggleRow; 3],
}

impl InstancesPage {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        self.toggles
            .iter()
            .filter(|toggle| toggle.enabled)
            .find(|toggle| toggle.line.rect.contains(point))
            .map(|toggle| toggle.control)
    }
}

pub(super) fn instances_page(
    column: &mut Column,
    strings: &Strings,
    preferences: &Preferences,
) -> InstancesPage {
    let section = column.caption(strings.instances_section);
    let toggles = NumberChoice::ALL.map(|choice| {
        let label = match choice {
            NumberChoice::Shown => strings.numbered_instances,
            NumberChoice::OnClose => strings.renumber_on_close,
            NumberChoice::OnMove => strings.renumber_on_move,
        };
        let state = choice.state(preferences.numbering);
        column.toggle_row(label, state, SettingControl::Numbers(choice))
    });
    InstancesPage { section, toggles }
}
