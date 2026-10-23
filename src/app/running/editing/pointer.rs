use winit::event::ElementState;

use trame::preferences::FormField;
use trame::ui::Hit;
use trame::ui::frame::SettingControl;

use super::Running;

impl Running {
    pub(in super::super) fn editing_button(&mut self, state: ElementState) -> bool {
        if self.command_field.is_some() {
            return self.command_button(state);
        }
        if self.editing.is_none() {
            return false;
        }
        let pressed = state == ElementState::Pressed;
        if !pressed && self.drag_released() {
            self.changed();
            return true;
        }
        let hit = self.hit_under_pointer();
        match hit {
            Some(Hit::EditSwatch(color)) if pressed => self.pick_color(color),
            Some(Hit::EditEmblem(emblem)) if pressed => self.pick_emblem(emblem),
            Some(Hit::FormField(field)) if pressed => self.focus_form(field),
            Some(Hit::FormButton(action)) if pressed => self.form_action(action),
            Some(Hit::List(control)) if pressed => self.list_control(control),
            Some(Hit::EditClose) if pressed => self.cancel_edit(),
            Some(
                Hit::EditField
                | Hit::EditStrip
                | Hit::EditSwatch(_)
                | Hit::EditEmblem(_)
                | Hit::EditClose
                | Hit::FormField(_)
                | Hit::FormButton(_)
                | Hit::List(_)
                | Hit::Dialog,
            ) => return true,
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

    pub(in crate::app::running) fn focus_form(&mut self, field: FormField) {
        if let Some(form) = self.connect_form_mut() {
            form.focus_on(field);
        }
    }

    fn command_button(&mut self, state: ElementState) -> bool {
        if self.hit_under_pointer() == Some(Hit::Setting(SettingControl::StartupCommand)) {
            return true;
        }
        if state == ElementState::Pressed {
            self.commit_edit();
            self.changed();
        }
        false
    }

    pub(in crate::app::running) fn hit_under_pointer(&self) -> Option<Hit> {
        self.pointer
            .position
            .and_then(|point| self.frame.hit(point, false))
    }
}
