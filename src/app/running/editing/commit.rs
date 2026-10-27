use trame::theme::{Emblem, Letters, Swatch};
use trame::ui::frame::{EditTarget, FormAction, SettingControl};

use super::{EditMode, Running};

impl Running {
    pub(super) fn commit_edit(&mut self) {
        if let Some((control, field)) = self.setting_field.take() {
            let next = match control {
                SettingControl::HistoryLines => self.preferences.typed_history_lines(field.text()),
                _ => self.preferences.with_startup_command(field.text()),
            };
            return self.set_preferences(next);
        }
        if self.settings_form_open() {
            return self.save_settings_form();
        }
        let Some(mut editing) = self.editing.take() else {
            return;
        };
        match &mut editing.mode {
            EditMode::Text { field, original } => {
                self.rename(editing.target, field.text(), original)
            }
            EditMode::Form(panel) => {
                match panel.form.as_mut().and_then(|entry| self.store_host(entry)) {
                    Some(host) => self.connect(editing.target, host),
                    None => self.editing = Some(editing),
                }
            }
            EditMode::Folder { original, chosen } => {
                if let Some(folder) = chosen
                    .take()
                    .filter(|folder| Some(folder) != original.as_ref())
                {
                    self.folder_picked(editing.target.into(), folder);
                }
            }
            EditMode::Picker(picker) => self.apply_color(editing.target, picker),
            EditMode::Emblem { letters } => match Letters::parse(letters.text()) {
                Some(letters) => self.set_emblem(editing.target, Emblem::Letters(letters)),
                None => self.editing = Some(editing),
            },
            EditMode::Color { .. } => {}
        }
    }

    pub(in crate::app::running) fn form_action(&mut self, action: FormAction) {
        match action {
            FormAction::Connect | FormAction::Save | FormAction::Apply => self.commit_edit(),
            FormAction::Change => self.change_chosen_folder(),
            FormAction::Add => self.add_color(),
            FormAction::Image => self.ask_image(),
            FormAction::Cancel => self.step_back(),
            FormAction::Close | FormAction::Restart => self.cancel_edit(),
        }
    }

    fn rename(&mut self, target: EditTarget, text: &str, original: &str) {
        if text == original {
            return;
        }
        match target {
            EditTarget::Tab(id) => self.workspace.rename_tab(id, text),
            EditTarget::Instance(id) => self.workspace.rename_instance(id, text),
        }
    }

    pub(in crate::app::running) fn connect_listed(&mut self, index: usize) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        if let EditMode::Form(panel) = editing.mode
            && let Some(host) = panel.hosts.into_iter().nth(index)
        {
            self.connect(editing.target, host);
        }
    }

    pub(super) fn cancel_edit(&mut self) {
        self.setting_field = None;
        if self.close_settings_form() {
            return;
        }
        self.editing = None;
    }

    pub(super) fn pick_color(&mut self, color: Option<Swatch>) {
        if let Some(editing) = self.editing.take() {
            self.paint_target(editing.target, color);
        }
    }

    pub(super) fn paint_target(&mut self, target: EditTarget, color: Option<Swatch>) {
        match target {
            EditTarget::Tab(id) => self.workspace.set_tab_color(id, color),
            EditTarget::Instance(id) => self.workspace.set_instance_color(id, color),
        }
    }

    pub(super) fn pick_emblem(&mut self, emblem: Emblem) {
        if let Some(editing) = self.editing.take() {
            self.set_emblem(editing.target, emblem);
        }
    }
}
