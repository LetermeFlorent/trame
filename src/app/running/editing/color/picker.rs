use trame::input::TextField;
use trame::theme::{Hsv, Rgba, hex_code, parse_hex};
use trame::ui::frame::PickerView;

use crate::app::running::editing::{EditMode, Editing};

pub(in crate::app::running::editing) struct Picker {
    pub(in crate::app::running::editing) slot: Option<usize>,
    pub(in crate::app::running::editing) hsv: Hsv,
    field: TextField,
    pub(in crate::app::running::editing) first: usize,
}

impl Picker {
    pub(in crate::app::running::editing) fn new(
        slot: Option<usize>,
        rgb: u32,
        first: usize,
    ) -> Self {
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

    pub(in crate::app::running::editing) fn view(&self) -> PickerView<'_> {
        PickerView {
            hsv: self.hsv,
            field: &self.field,
            editing: self.slot.is_some(),
        }
    }

    pub(in crate::app::running::editing) fn edit_code(
        &mut self,
        change: impl FnOnce(&mut TextField),
    ) {
        change(&mut self.field);
        if let Some(rgb) = parse_hex(self.field.text()) {
            self.hsv = Hsv::from_rgb(rgb);
        }
    }

    pub(in crate::app::running::editing) fn show_code(&mut self) {
        self.field = TextField::new(&hex_code(self.hsv.to_rgb()));
    }
}

impl Editing {
    pub(in crate::app::running::editing) fn picker_mut(&mut self) -> Option<&mut Picker> {
        match &mut self.mode {
            EditMode::Picker(picker) => Some(picker),
            _ => None,
        }
    }
}

pub(in crate::app::running::editing) fn rgb_of(color: Rgba) -> u32 {
    let [r, g, b] = color.to_rgb8();
    u32::from_be_bytes([0, r, g, b])
}
