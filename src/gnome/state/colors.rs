use trame::input::TextField;
use trame::theme::{Hsv, MAX_LETTERS, Rgba, hex_code, parse_hex};
use trame::ui::Hit;
use trame::ui::frame::{ListControl, PickerPart, PickerView};

use super::WindowState;
use super::editing::{EditMode, Editing};

pub struct Picker {
    pub slot: Option<usize>,
    pub hsv: Hsv,
    field: TextField,
    pub first: usize,
}

impl Picker {
    pub fn new(slot: Option<usize>, rgb: u32, first: usize) -> Self {
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

    pub fn view(&self) -> PickerView<'_> {
        PickerView {
            hsv: self.hsv,
            field: &self.field,
            editing: self.slot.is_some(),
        }
    }

    pub fn edit_code(&mut self, change: impl FnOnce(&mut TextField)) {
        change(&mut self.field);
        if let Some(rgb) = parse_hex(self.field.text()) {
            self.hsv = Hsv::from_rgb(rgb);
        }
    }

    pub fn show_code(&mut self) {
        self.field = TextField::new(&hex_code(self.hsv.to_rgb()));
    }
}

pub fn rgb_of(color: Rgba) -> u32 {
    let [r, g, b] = color.to_rgb8();
    u32::from_be_bytes([0, r, g, b])
}

pub fn edit_letters(field: &mut TextField, change: impl FnOnce(&mut TextField)) {
    let before = field.clone();
    change(field);
    if field.text().chars().count() > MAX_LETTERS {
        *field = before;
    }
}

impl WindowState {
    pub fn custom_control(&mut self, control: ListControl) -> bool {
        let Some(EditMode::Color { first }) = self.editing.as_ref().map(|e| &e.mode) else {
            return false;
        };
        let first = *first;
        let listed = |i: usize| self.desk.preferences.custom_colors.get(i).copied();
        match control {
            ListControl::Pick(i) => {
                if let Some(color) = listed(i) {
                    self.pick_color(Some(color));
                }
            }
            ListControl::Change(i) => {
                if let Some(color) = listed(i) {
                    let rgb = rgb_of(self.desk.theme.swatch(color));
                    self.open_picker(Picker::new(Some(i), rgb, first));
                }
            }
            ListControl::Forget(i) => {
                let effects = self.desk.forget_custom_color(i);
                self.apply(effects);
            }
            ListControl::Thumb | ListControl::Track => return false,
        }
        true
    }

    pub fn add_color(&mut self) {
        let Some(editing) = &self.editing else {
            return;
        };
        let EditMode::Color { first } = editing.mode else {
            return;
        };
        let color = self.desk.color_of(editing.target);
        let shown = color.map_or(self.desk.theme.accent, |c| self.desk.theme.swatch(c));
        self.open_picker(Picker::new(None, rgb_of(shown), first));
    }

    fn open_picker(&mut self, picker: Picker) {
        if let Some(editing) = self.editing.as_mut() {
            editing.mode = EditMode::Picker(picker);
        }
    }

    pub fn step_back(&mut self) {
        let first = self
            .editing
            .as_mut()
            .and_then(Editing::picker_mut)
            .map(|p| p.first);
        match first {
            Some(first) => {
                if let Some(editing) = self.editing.as_mut() {
                    editing.mode = EditMode::Color { first };
                }
            }
            None => self.cancel_edit(),
        }
    }

    pub fn press_picker(&mut self, part: PickerPart) {
        if part != PickerPart::Field {
            self.drag_picker(part);
            self.drag_pressed(Hit::Picker(part));
        }
    }

    pub fn drag_picker(&mut self, part: PickerPart) {
        let frame = self.frame.editor.as_ref().and_then(|e| e.picker.as_ref());
        let picker = self.editing.as_mut().and_then(Editing::picker_mut);
        if let (Some(point), Some(frame), Some(picker)) = (self.pointer, frame, picker) {
            picker.hsv = frame.picked(part, point, picker.hsv);
            picker.show_code();
        }
    }
}
