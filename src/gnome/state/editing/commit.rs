use trame::desk::Edit;
use trame::preferences::FormField;
use trame::theme::{Emblem, Letters, Swatch};
use trame::ui::frame::{EditTarget, FormAction, SettingControl};

use super::{EditMode, Editing};
use crate::gnome::state::{PickerJob, WindowState};

impl WindowState {
    pub fn commit_edit(&mut self) {
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
        let Some(editing) = self.editing.take() else {
            return;
        };
        self.finish_editing(editing);
    }

    fn finish_editing(&mut self, mut editing: Editing) {
        let edit = match &mut editing.mode {
            EditMode::Text { field, original } => Some(Edit::Name {
                text: field.text().to_owned(),
                original: original.clone(),
            }),
            EditMode::Folder { original, chosen } => chosen
                .take()
                .filter(|f| Some(f) != original.as_ref())
                .map(Edit::Folder),
            EditMode::Picker(p) => Some(Edit::CustomColor {
                slot: p.slot,
                color: Swatch::custom(p.hsv.to_rgb()),
            }),
            EditMode::Emblem { letters } => match Letters::parse(letters.text()) {
                Some(l) => Some(Edit::Emblem(Emblem::Letters(l))),
                None => return self.keep_editing(editing),
            },
            EditMode::Form(panel) => match panel.form.as_mut().and_then(|e| self.store_host(e)) {
                Some(host) => Some(Edit::Host(host)),
                None => return self.keep_editing(editing),
            },
            EditMode::Color { .. } => None,
        };
        if let Some(edit) = edit {
            let effects = self.desk.commit(editing.target, edit);
            self.apply(effects);
        }
    }

    fn keep_editing(&mut self, editing: Editing) {
        self.editing = Some(editing);
    }

    pub fn form_action(&mut self, action: FormAction) {
        match action {
            FormAction::Connect | FormAction::Save | FormAction::Apply => self.commit_edit(),
            FormAction::Change => self.request_folder_picker(),
            FormAction::Add => self.add_color(),
            FormAction::Image => self.request_image_picker(),
            FormAction::Cancel => self.step_back(),
            _ => self.cancel_edit(),
        }
    }

    fn request_folder_picker(&mut self) {
        if let Some(Editing {
            target,
            mode: EditMode::Folder { chosen, .. },
            ..
        }) = &self.editing
        {
            self.picker_req = Some(PickerJob::Folder((*target).into(), chosen.clone()));
        }
    }

    fn request_image_picker(&mut self) {
        let target = self.editing.as_ref().map(|e| e.target);
        let dir = self.desk.icons.dir().map(ToOwned::to_owned);
        if let (Some(EditTarget::Instance(id)), Some(dir)) = (target, dir) {
            self.picker_req = Some(PickerJob::Image(id, dir));
        }
    }

    pub fn cancel_edit(&mut self) {
        self.setting_field = None;
        if !self.close_settings_form() {
            self.editing = None;
        }
    }

    pub fn pick_color(&mut self, color: Option<Swatch>) {
        self.commit_taken(Edit::Color(color));
    }

    pub fn pick_emblem(&mut self, emblem: Emblem) {
        self.commit_taken(Edit::Emblem(emblem));
    }

    fn commit_taken(&mut self, edit: Edit) {
        if let Some(editing) = self.editing.take() {
            let effects = self.desk.commit(editing.target, edit);
            self.apply(effects);
        }
    }

    pub fn toggle_fill(&mut self) {
        if let Some(target) = self.editing.as_ref().map(|e| e.target) {
            let on = !self.desk.fill_of(target);
            let effects = self.desk.commit(target, Edit::Fill(on));
            self.apply(effects);
        }
    }

    pub fn focus_form(&mut self, field: FormField) {
        if let Some(form) = self.connect_form_mut() {
            form.focus_on(field);
        }
    }
}
