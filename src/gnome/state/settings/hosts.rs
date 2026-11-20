use trame::desk::{Closing, Edit, HostForm, written_by_hand};
use trame::preferences::{ConnectForm, SshHost};
use trame::ui::Hit;
use trame::ui::frame::ListControl;

use super::{HostsPanel, ListKind};
use crate::gnome::state::WindowState;
use crate::gnome::state::editing::{EditMode, Editing};

impl WindowState {
    pub fn panel_mut(&mut self) -> Option<&mut HostsPanel> {
        if let Some(editing) = self.editing.as_mut()
            && let Some(panel) = editing.panel_mut()
        {
            return Some(panel);
        }
        self.settings.as_mut()?.hosts.as_mut()
    }

    pub fn connect_form_mut(&mut self) -> Option<&mut ConnectForm> {
        Some(&mut self.panel_mut()?.form.as_mut()?.form)
    }

    pub fn settings_form_open(&self) -> bool {
        self.settings
            .as_ref()
            .and_then(|s| s.hosts.as_ref())
            .is_some_and(|p| p.form.is_some())
    }

    pub fn close_settings_form(&mut self) -> bool {
        self.settings
            .as_mut()
            .and_then(|s| s.hosts.as_mut())
            .is_some_and(|p| p.form.take().is_some())
    }

    pub fn store_host(&mut self, entry: &mut HostForm) -> Option<SshHost> {
        let out = self.desk.store_host(entry);
        self.apply(out.effects);
        out.value
    }

    pub fn save_settings_form(&mut self) {
        let Some(mut entry) = self
            .settings
            .as_mut()
            .and_then(|s| s.hosts.as_mut())
            .and_then(|p| p.form.take())
        else {
            return;
        };
        if self.store_host(&mut entry).is_some() {
            return self.reload_hosts();
        }
        if let Some(panel) = self.panel_mut() {
            panel.form = Some(entry);
        }
    }

    pub fn reload_hosts(&mut self) {
        let hosts = self.desk.known_hosts();
        if let Some(panel) = self.panel_mut() {
            panel.hosts = hosts;
        }
        self.changed();
    }

    pub fn list_control(&mut self, control: ListControl) {
        if self.custom_control(control) || self.shortcut_control(control) {
            return self.changed();
        }
        match control {
            ListControl::Pick(i) => self.pick_host(i),
            ListControl::Change(i) => {
                if let Some(panel) = self.panel_mut()
                    && let Some(host) = panel.hosts.get(i).cloned()
                {
                    panel.form = Some(HostForm::changing(host));
                }
            }
            ListControl::Forget(i) => self.forget_host_at(i),
            ListControl::Thumb => self.drag_pressed(Hit::List(control)),
            ListControl::Track => self.page_list(ListKind::Hosts),
        }
        self.changed();
    }

    fn pick_host(&mut self, index: usize) {
        if let Some(Editing {
            target,
            mode: EditMode::Form(p),
            ..
        }) = self.editing.take()
            && let Some(host) = p.hosts.into_iter().nth(index)
        {
            let effects = self.desk.commit(target, Edit::Host(host));
            self.apply(effects);
        }
    }

    fn forget_host_at(&mut self, index: usize) {
        let Some(host) = self.panel_mut().and_then(|p| p.hosts.get(index).cloned()) else {
            return;
        };
        if host.alias().is_some() && written_by_hand(&host) {
            self.desk.closing = Some(Closing::Forget(host));
        } else {
            let effects = self.desk.forget_host(&host);
            self.apply(effects);
            self.reload_hosts();
        }
    }
}
