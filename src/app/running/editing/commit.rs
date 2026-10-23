use trame::preferences::SshHost;
use trame::theme::{Emblem, Swatch};
use trame::ui::frame::EditTarget;

use super::{EditMode, Running, TextPurpose};

impl Running {
    pub(super) fn commit_edit(&mut self) {
        if let Some(field) = self.command_field.take() {
            let next = self.preferences.with_startup_command(field.text());
            return self.set_preferences(next);
        }
        let Some(editing) = self.editing.take() else {
            return;
        };
        let EditMode::Text { field, purpose } = &editing.mode else {
            return;
        };
        let text = field.text();
        match (purpose, editing.target) {
            (TextPurpose::Rename(original), _) if text == original => {}
            (TextPurpose::Rename(_), EditTarget::Tab(id)) => self.workspace.rename_tab(id, text),
            (TextPurpose::Rename(_), EditTarget::Instance(id)) => {
                self.workspace.rename_instance(id, text);
            }
            (TextPurpose::Connect, target) => match SshHost::parse(text) {
                Some(host) => self.connect(target, host),
                None => self.editing = Some(editing),
            },
        }
    }

    pub(super) fn cancel_edit(&mut self) {
        self.command_field = None;
        self.editing = None;
    }

    pub(super) fn pick_color(&mut self, color: Option<Swatch>) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        match editing.target {
            EditTarget::Tab(id) => self.workspace.set_tab_color(id, color),
            EditTarget::Instance(id) => self.workspace.set_instance_color(id, color),
        }
    }

    pub(super) fn pick_emblem(&mut self, emblem: Emblem) {
        if let Some(EditTarget::Instance(id)) = self.editing.take().map(|editing| editing.target) {
            self.workspace.set_instance_emblem(id, emblem);
        }
    }
}
