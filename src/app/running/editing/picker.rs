use trame::input::TextField;
use trame::theme::{Hsv, Rgba, hex_code, parse_hex};
use trame::ui::frame::PickerView;

use super::{EditMode, Editing};

pub(super) struct Picker {
    pub(super) slot: Option<usize>,
    pub(super) hsv: Hsv,
    field: TextField,
    pub(super) first: usize,
}

impl Picker {
    pub(super) fn new(slot: Option<usize>, rgb: u32, first: usize) -> Self {
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

    pub(super) fn show_code(&mut self) {
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

pub(super) fn rgb_of(color: Rgba) -> u32 {
    let [r, g, b] = color.to_rgb8();
    u32::from_be_bytes([0, r, g, b])
}
