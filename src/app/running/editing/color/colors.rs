use trame::ui::Hit;
use trame::ui::frame::{ListControl, PickerPart};

use super::picker::{Picker, rgb_of};
use crate::app::running::Running;
use crate::app::running::editing::{EditMode, Editing};

impl Running {
    pub(in crate::app::running) fn custom_control(&mut self, control: ListControl) -> bool {
        let Some(EditMode::Color { first }) = self.editing.as_ref().map(|editing| &editing.mode)
        else {
            return false;
        };
        let first = *first;
        let listed = |index: usize| self.desk.preferences.custom_colors.get(index).copied();
        match control {
            ListControl::Pick(index) => {
                if let Some(color) = listed(index) {
                    self.pick_color(Some(color));
                }
            }
            ListControl::Change(index) => {
                if let Some(color) = listed(index) {
                    let rgb = rgb_of(self.desk.theme.swatch(color));
                    self.open_picker(Picker::new(Some(index), rgb, first));
                }
            }
            ListControl::Forget(index) => {
                let effects = self.desk.forget_custom_color(index);
                self.apply(effects);
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
            return match self.panel_mut().is_some() {
                true => Some(&mut self.panel_mut()?.first),
                false => Some(&mut self.keys_view()?.first),
            };
        }
        match &mut self.editing.as_mut()?.mode {
            EditMode::Color { first } => Some(first),
            _ => None,
        }
    }

    pub(in crate::app::running::editing) fn add_color(&mut self) {
        let Some(editing) = &self.editing else {
            return;
        };
        let EditMode::Color { first } = editing.mode else {
            return;
        };
        let color = self.desk.color_of(editing.target);
        let shown = color.map_or(self.desk.theme.accent, |color| {
            self.desk.theme.swatch(color)
        });
        self.open_picker(Picker::new(None, rgb_of(shown), first));
    }

    fn open_picker(&mut self, picker: Picker) {
        if let Some(editing) = self.editing.as_mut() {
            editing.mode = EditMode::Picker(picker);
        }
    }

    pub(in crate::app::running::editing) fn step_back(&mut self) {
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

    pub(in crate::app::running::editing) fn press_picker(&mut self, part: PickerPart) {
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
