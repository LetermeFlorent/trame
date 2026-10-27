mod editor;
mod label;
mod place;

use crate::geometry::{Point, Rect};

use super::{ChoiceRow, HostList, SettingControl, SettingLine, control_under};
pub use label::key_label;
pub(super) use place::shortcuts_page;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyControl {
    Add,
    Remove(usize),
    Reset,
    ResetAll,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyNotice {
    Taken(usize),
    Typing,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShortcutsView {
    pub first: usize,
    pub action: Option<usize>,
    pub capturing: bool,
    pub notice: Option<KeyNotice>,
}

#[derive(Clone, Debug)]
pub struct KeyDetail {
    pub line: SettingLine,
    pub changed: bool,
}

#[derive(Clone, Debug)]
pub struct KeyLine {
    pub line: SettingLine,
    pub remove: Rect,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct ShortcutsPage {
    pub header: ChoiceRow,
    pub list: Option<HostList>,
    pub details: Vec<KeyDetail>,
    pub keys: Vec<KeyLine>,
    pub hint: Option<SettingLine>,
    pub capturing: bool,
    pub footer: Option<ChoiceRow>,
    pub notice: Option<SettingLine>,
}

impl ShortcutsPage {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let removes = self.keys.iter().map(|key| {
            let control = SettingControl::Keys(KeyControl::Remove(key.index));
            (key.remove, control)
        });
        let controls = self
            .header
            .controls()
            .chain(self.footer.iter().flat_map(ChoiceRow::controls))
            .chain(removes);
        control_under(point, controls)
    }
}
