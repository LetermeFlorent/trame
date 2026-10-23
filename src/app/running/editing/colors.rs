use trame::input::TextField;
use trame::theme::{Hsv, Rgba, Swatch, hex_code, parse_hex};
use trame::ui::Hit;
use trame::ui::frame::{EditTarget, ListControl, PickerPart, PickerView};

use super::{EditMode, Editing, Running};

pub(super) struct Picker {
    slot: Option<usize>,
    hsv: Hsv,
    field: TextField,
    first: usize,
}

impl Picker {
    fn new(slot: Option<usize>, rgb: u32, first: usize) -> Self {
        let mut picker = Self {
            slot,
            hsv: Hsv::from_rgb(rgb),
            field: TextField::new(""),
            first,
        };
        picker.show_code();
        picker.field.select_all();
        picker
    }

    pub(super) fn view(&self) -> PickerView<'_> {
        PickerView {
            hsv: self.hsv,
            field: &self.field,
            editing: self.slot.is_some(),
        }
    }

    pub(super) fn edit_code(&mut self, change: impl FnOnce(&mut TextField)) {
        change(&mut self.field);
        if let Some(rgb) = parse_hex(self.field.text()) {
            self.hsv = Hsv::from_rgb(rgb);
        }
    }

    fn show_code(&mut self) {
        self.field = TextField::new(&hex_code(self.hsv.to_rgb()));
    }
}

impl Editing {
    pub(super) fn picker_mut(&mut self) -> Option<&mut Picker> {
        match &mut self.mode {
            EditMode::Picker(picker) => Some(picker),
            _ => None,
        }
    }
}

fn rgb_of(color: Rgba) -> u32 {
    let [r, g, b] = color.to_rgb8();
    u32::from_be_bytes([0, r, g, b])
}

impl Running {
    pub(in crate::app::running) fn custom_control(&mut self, control: ListControl) -> bool {
        let Some(EditMode::Color { first }) = self.editing.as_ref().map(|editing| &editing.mode)
        else {
            return false;
        };
        let first = *first;
        let listed = |index: usize| self.preferences.custom_colors.get(index).copied();
        match control {
            ListControl::Pick(index) => {
                if let Some(color) = listed(index) {
                    self.pick_color(Some(color));
                }
            }
            ListControl::Change(index) => {
                if let Some(color) = listed(index) {
                    let rgb = rgb_of(self.theme.swatch(color));
                    self.open_picker(Picker::new(Some(index), rgb, first));
                }
            }
            ListControl::Forget(index) => {
                let next = self.preferences.without_custom_color(index);
                self.set_preferences(next);
            }
            ListControl::Thumb | ListControl::Track => return false,
        }
        true
    }

    pub(in crate::app::running) fn list_first_mut(&mut self) -> Option<&mut usize> {
        let colors = matches!(
            self.editing,
            Some(Editing {
                mode: EditMode::Color { .. },
                ..
            })
        );
        if !colors {
            return Some(&mut self.panel_mut()?.first);
        }
        match &mut self.editing.as_mut()?.mode {
            EditMode::Color { first } => Some(first),
            _ => None,
        }
    }

    pub(super) fn add_color(&mut self) {
        let Some(editing) = &self.editing else {
            return;
        };
        let EditMode::Color { first } = editing.mode else {
            return;
        };
        let color = self.color_of(editing.target);
        let shown = color.map_or(self.theme.accent, |color| self.theme.swatch(color));
        self.open_picker(Picker::new(None, rgb_of(shown), first));
    }

    fn open_picker(&mut self, picker: Picker) {
        if let Some(editing) = self.editing.as_mut() {
            editing.mode = EditMode::Picker(picker);
        }
    }

    pub(super) fn apply_color(&mut self, target: EditTarget, picker: &Picker) {
        let color = Swatch::custom(picker.hsv.to_rgb());
        let custom = &self.preferences.custom_colors;
        let old = picker.slot.and_then(|slot| custom.get(slot).copied());
        let next = self.preferences.with_custom_color(picker.slot, color);
        self.set_preferences(next);
        if let Some(old) = old.filter(|old| *old != color) {
            self.workspace.replace_color(old, color);
        }
        self.paint_target(target, Some(color));
    }

    pub(super) fn step_back(&mut self) {
        let picker = self.editing.as_mut().and_then(Editing::picker_mut);
        match picker.map(|picker| picker.first) {
            Some(first) => self.open_colors(first),
            None => self.cancel_edit(),
        }
    }

    fn open_colors(&mut self, first: usize) {
        if let Some(editing) = self.editing.as_mut() {
            editing.mode = EditMode::Color { first };
        }
    }

    pub(super) fn press_picker(&mut self, part: PickerPart) {
        if part != PickerPart::Field {
            self.drag_picker(part);
            self.drag_pressed(Hit::Picker(part));
        }
    }

    pub(in crate::app::running) fn drag_picker(&mut self, part: PickerPart) {
        let frame = self
            .frame
            .editor
            .as_ref()
            .and_then(|editor| editor.picker.as_ref());
        let picker = self.editing.as_mut().and_then(Editing::picker_mut);
        if let (Some(point), Some(frame), Some(picker)) = (self.pointer.position, frame, picker) {
            picker.hsv = frame.picked(part, point, picker.hsv);
            picker.show_code();
        }
    }
}
