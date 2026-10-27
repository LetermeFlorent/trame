use crate::geometry::{Point, Rect};
use crate::ui::frame::FieldFrame;

use super::{ChoiceRow, FormPage, SettingControl, Stepper, ToggleRow};

impl FormPage {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        let field = self
            .field
            .as_ref()
            .map(|(control, field)| (field.rect, *control));
        let toggles = self
            .toggles
            .iter()
            .filter(|toggle| toggle.enabled)
            .map(|toggle| (toggle.line.rect, toggle.control));
        let steppers = self.steppers.iter().flat_map(Stepper::targets);
        let rows = self.rows.iter().flat_map(ChoiceRow::controls);
        field
            .into_iter()
            .chain(toggles)
            .chain(steppers)
            .chain(rows)
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, control)| control)
    }

    pub fn field_for(&self, wanted: SettingControl) -> Option<&FieldFrame> {
        self.field
            .as_ref()
            .filter(|(control, _)| *control == wanted)
            .map(|(_, field)| field)
    }

    pub fn toggle(&self, control: SettingControl) -> Option<&ToggleRow> {
        self.toggles.iter().find(|toggle| toggle.control == control)
    }

    pub fn stepper(&self, control: SettingControl) -> Option<&Stepper> {
        self.steppers
            .iter()
            .find(|stepper| stepper.controls.contains(&control) || stepper.typed == Some(control))
    }

    pub fn row(&self, control: SettingControl) -> Option<&ChoiceRow> {
        self.rows
            .iter()
            .find(|row| row.controls().any(|(_, own)| own == control))
    }
}

impl Stepper {
    pub fn targets(&self) -> impl Iterator<Item = (Rect, SettingControl)> + '_ {
        let [shrink, grow] = self.controls;
        [(self.smaller, shrink), (self.bigger, grow)]
            .into_iter()
            .chain(self.typed.map(|control| (self.value_rect, control)))
    }
}
