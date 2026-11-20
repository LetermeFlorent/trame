use trame::ui::frame::{PickerPart, SettingControl};
use trame::ui::{Hit, ResizeEdge};
use trame::workspace::{Axis, Divider};

use crate::gnome::state::WindowState;
use crate::gnome::state::drag::Drag;

impl WindowState {
    pub(super) fn editing_button(&mut self, pressed: bool) -> bool {
        let hit = self.pointer.and_then(|p| self.frame.hit(p, false));
        if let Some(done) = self.setting_field_button(pressed, hit) {
            return done;
        }
        if self.editing.is_none() {
            return false;
        }
        if !pressed && self.drag_released() {
            self.changed();
            return true;
        }
        self.apply_editing_hit(pressed, hit)
    }

    fn setting_field_button(&mut self, pressed: bool, hit: Option<Hit>) -> Option<bool> {
        let (control, _) = self.setting_field?;
        if hit == Some(Hit::Setting(control)) {
            return Some(true);
        }
        if pressed {
            self.commit_edit();
            self.changed();
        }
        Some(false)
    }

    fn apply_editing_hit(&mut self, pressed: bool, hit: Option<Hit>) -> bool {
        match hit {
            Some(Hit::EditSwatch(c)) if pressed => self.pick_color(c),
            Some(Hit::EditEmblem(e)) if pressed => self.pick_emblem(e),
            Some(Hit::FormField(f)) if pressed => self.focus_form(f),
            Some(Hit::FormButton(a)) if pressed => self.form_action(a),
            Some(Hit::List(c)) if pressed => self.list_control(c),
            Some(Hit::EditClose) if pressed => self.step_back(),
            Some(Hit::Setting(SettingControl::Fill)) if pressed => self.toggle_fill(),
            Some(Hit::Picker(p)) if pressed => self.press_picker(p),
            Some(h) if inside_editor(h) => return true,
            _ if pressed => {
                self.cancel_edit();
                self.changed();
                return false;
            }
            _ => return false,
        }
        self.changed();
        true
    }

    pub(super) fn drag_cursor_name(&self) -> &'static str {
        match &self.drag {
            Some(Drag::Divider(d)) => axis_cursor(d.axis),
            Some(Drag::Sidebar(_)) => "col-resize",
            Some(Drag::Tab(..) | Drag::Instance(..)) => "grabbing",
            _ => "default",
        }
    }
}

fn inside_editor(hit: Hit) -> bool {
    matches!(
        hit,
        Hit::EditField
            | Hit::EditStrip
            | Hit::EditSwatch(_)
            | Hit::EditEmblem(_)
            | Hit::EditClose
            | Hit::Picker(_)
            | Hit::FormField(_)
            | Hit::FormButton(_)
            | Hit::List(_)
            | Hit::Dialog
    )
}

fn axis_cursor(axis: Axis) -> &'static str {
    match axis {
        Axis::Row => "col-resize",
        Axis::Column => "row-resize",
    }
}

pub(super) fn cursor_name(hit: Option<Hit>, dividers: &[Divider]) -> &'static str {
    match hit {
        Some(Hit::Resize(ResizeEdge::North | ResizeEdge::South)) => "ns-resize",
        Some(Hit::Resize(ResizeEdge::East | ResizeEdge::West)) => "ew-resize",
        Some(Hit::Resize(ResizeEdge::NorthEast | ResizeEdge::SouthWest)) => "nesw-resize",
        Some(Hit::Resize(ResizeEdge::NorthWest | ResizeEdge::SouthEast)) => "nwse-resize",
        Some(Hit::Divider(i)) => dividers.get(i).map_or("default", |d| axis_cursor(d.axis)),
        Some(Hit::SidebarEdge) => "col-resize",
        Some(
            Hit::Terminal(_) | Hit::EditField | Hit::FormField(_) | Hit::Picker(PickerPart::Field),
        ) => "text",
        None
        | Some(
            Hit::TitleBar
            | Hit::Sidebar
            | Hit::TabBar(_)
            | Hit::EditStrip
            | Hit::SettingsPanel
            | Hit::Menu
            | Hit::Dialog
            | Hit::Backdrop,
        ) => "default",
        Some(_) => "pointer",
    }
}
