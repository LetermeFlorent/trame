use winit::event::ElementState;

use trame::preferences::FormField;
use trame::ui::Hit;
use trame::ui::frame::SettingControl;

use super::{EditMode, Editing, Running};

impl Running {
    pub(in super::super) fn editing_button(&mut self, state: ElementState) -> bool {
        if self.command_field.is_some() {
            return self.command_button(state);
        }
        if self.editing.is_none() {
            return false;
        }
        let hit = self.hit_under_pointer();
        let pressed = state == ElementState::Pressed;
        match hit {
            Some(Hit::EditSwatch(color)) if pressed => self.pick_color(color),
            Some(Hit::EditEmblem(emblem)) if pressed => self.pick_emblem(emblem),
            Some(Hit::FormField(field)) if pressed => self.focus_form(field),
            Some(Hit::FormConnect) if pressed => self.commit_edit(),
            Some(Hit::EditHost(index)) if pressed => self.connect_listed(index),
            Some(Hit::EditClose) if pressed => self.cancel_edit(),
            Some(
                Hit::EditField
                | Hit::EditStrip
                | Hit::EditSwatch(_)
                | Hit::EditEmblem(_)
                | Hit::EditHost(_)
                | Hit::EditClose
                | Hit::FormField(_)
                | Hit::FormConnect,
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

    fn focus_form(&mut self, field: FormField) {
        if let Some(Editing {
            mode: EditMode::Form(form, _),
            ..
        }) = &mut self.editing
        {
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

    fn hit_under_pointer(&self) -> Option<Hit> {
        self.pointer
            .position
            .and_then(|point| self.frame.hit(point, false))
    }
}
