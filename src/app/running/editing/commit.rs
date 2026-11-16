use trame::desk::Edit;
use trame::theme::{Emblem, Swatch};
use trame::ui::frame::{EditTarget, FormAction, SettingControl};

use super::ending::Ending;
use super::{EditMode, Running};

impl Running {
    pub(super) fn commit_edit(&mut self) {
        if let Some((control, field)) = self.setting_field.take() {
            let next = match control {
                SettingControl::HistoryLines => {
                    self.desk.preferences.typed_history_lines(field.text())
                }
                _ => self.desk.preferences.with_startup_command(field.text()),
            };
            return self.set_preferences(next);
        }
        if self.settings_form_open() {
            return self.save_settings_form();
        }
        let Some(mut editing) = self.editing.take() else {
            return;
        };
        let ending = match &mut editing.mode {
            EditMode::Form(panel) => {
                match panel.form.as_mut().and_then(|entry| self.store_host(entry)) {
                    Some(host) => Ending::Commit(Edit::Host(host)),
                    None => Ending::Keep,
                }
            }
            mode => mode.ending(),
        };
        match ending {
            Ending::Commit(edit) => self.commit(editing.target, edit),
            Ending::Keep => self.editing = Some(editing),
            Ending::Close => {}
        }
    }

    pub(in crate::app::running) fn form_action(&mut self, action: FormAction) {
        match action {
            FormAction::Connect | FormAction::Save | FormAction::Apply => self.commit_edit(),
            FormAction::Change => self.change_chosen_folder(),
            FormAction::Add => self.add_color(),
            FormAction::Image => self.ask_image(),
            FormAction::Cancel => self.step_back(),
            FormAction::Close | FormAction::Restart | FormAction::Remove | FormAction::Paste => {
                self.cancel_edit();
            }
        }
    }

    fn commit(&mut self, target: EditTarget, edit: Edit) {
        let effects = self.desk.commit(target, edit);
        self.apply(effects);
    }

    pub(in crate::app::running) fn connect_listed(&mut self, index: usize) {
        let Some(editing) = self.editing.take() else {
            return;
        };
        if let EditMode::Form(panel) = editing.mode
            && let Some(host) = panel.hosts.into_iter().nth(index)
        {
            self.commit(editing.target, Edit::Host(host));
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
            self.commit(editing.target, Edit::Color(color));
        }
    }

    pub(in crate::app::running) fn toggle_fill(&mut self) {
        let Some(target) = self.editing.as_ref().map(|editing| editing.target) else {
            return;
        };
        let on = !self.desk.fill_of(target);
        self.commit(target, Edit::Fill(on));
    }

    pub(super) fn pick_emblem(&mut self, emblem: Emblem) {
        if let Some(editing) = self.editing.take() {
            self.commit(editing.target, Edit::Emblem(emblem));
        }
    }
}
