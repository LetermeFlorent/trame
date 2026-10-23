use trame::preferences::{SshHost, vault};
use trame::theme::{Emblem, Swatch};
use trame::ui::frame::EditTarget;

use super::{EditMode, Running};
use crate::app::persist;

fn save_secret(host: &SshHost, secret: Option<&str>) {
    let Some(secret) = secret else {
        return;
    };
    if let Err(error) = vault::SSH.store(host, secret) {
        eprintln!(
            "trame: the password for {} could not be saved: {error}",
            host.label()
        );
    }
}

fn kept_in_config(host: SshHost) -> SshHost {
    let Some(config) = persist::ssh_config() else {
        return host;
    };
    config.keep(&host).unwrap_or_else(|error| {
        eprintln!(
            "trame: {} could not be added to the SSH config: {error}",
            host.label()
        );
        host
    })
}

impl Running {
    pub(super) fn commit_edit(&mut self) {
        if let Some(field) = self.command_field.take() {
            let next = self.preferences.with_startup_command(field.text());
            return self.set_preferences(next);
        }
        let Some(mut editing) = self.editing.take() else {
            return;
        };
        match &mut editing.mode {
            EditMode::Text { field, original } => {
                self.rename(editing.target, field.text(), original)
            }
            EditMode::Form(form) => match form.host() {
                Some(host) => {
                    let host = kept_in_config(host);
                    save_secret(&host, form.secret());
                    self.connect(editing.target, host);
                }
                None => {
                    if let Some(field) = form.first_invalid() {
                        form.focus_on(field);
                    }
                    self.editing = Some(editing);
                }
            },
            EditMode::Color | EditMode::Emblem => {}
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
