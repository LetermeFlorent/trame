mod settle;
mod updates;

use std::cell::RefCell;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use trame::desk::{MovedInstance, TornTab};
use trame::host::key::WindowKey;
use trame::host::pending::{Offer, PendingOffer, ShowsOffer};
use trame::host::persist::{Role, Saved, Slot, free_slot};
use trame::host::power::Process;
use trame::host::update::UpdateStatus;
use trame::log_error;
use trame::preferences::Preferences;
use trame::workspace::Snapshot;

use super::state::WindowState;

pub struct WindowHandle {
    pub key: WindowKey,
    pub window: adw::ApplicationWindow,
    pub state: Rc<RefCell<WindowState>>,
    pub refresh: Box<dyn Fn()>,
}

impl ShowsOffer for WindowHandle {
    fn show_offer(&mut self, offer: Offer) {
        let mut s = self.state.borrow_mut();
        let effects = s.desk.offer_update(offer);
        s.apply(effects);
        drop(s);
        (self.refresh)();
    }
}

pub enum OpenSpec {
    Normal(Option<Snapshot>, Option<PathBuf>),
    Torn(TornTab),
    Moved(MovedInstance),
}

pub type BuilderFn = Box<
    dyn Fn(
        WindowKey,
        Slot,
        Option<Preferences>,
        OpenSpec,
        &UpdateStatus,
    ) -> Result<WindowHandle, String>,
>;

pub struct Fleet {
    pub role: Role,
    pub saved: Vec<Saved>,
    pub opened: Option<PathBuf>,
    pub update: UpdateStatus,
    pub offer: PendingOffer,
    pub windows: Vec<WindowHandle>,
    pub process: Process,
    next_key: u32,
    pub(super) exiting: bool,
    builder: BuilderFn,
}

impl Fleet {
    pub fn new(
        role: Role,
        saved: Vec<Saved>,
        opened: Option<PathBuf>,
        update: UpdateStatus,
        builder: BuilderFn,
    ) -> Self {
        Self {
            role,
            saved,
            opened,
            update,
            offer: PendingOffer::default(),
            windows: Vec::new(),
            process: Process::default(),
            next_key: 0,
            exiting: false,
            builder,
        }
    }

    pub fn open_saved(&mut self, app: &adw::Application) {
        for item in mem::take(&mut self.saved) {
            let folder = self.opened.take();
            let spec = OpenSpec::Normal(item.snapshot, folder);
            if let Err(err) = self.spawn_window(item.slot, spec) {
                log_error!("a saved window could not open: {err}");
            }
        }
        if self.windows.is_empty() {
            app.quit();
        }
    }

    pub fn open_another(&mut self, folder: Option<PathBuf>) {
        if self.windows.is_empty() {
            return;
        }
        let slot = self.free_window_slot();
        if let Err(err) = self.spawn_window(slot, OpenSpec::Normal(None, folder)) {
            log_error!("a new window could not open: {err}");
        }
    }

    pub(super) fn free_window_slot(&self) -> usize {
        free_slot(
            self.windows
                .iter()
                .map(|w| w.state.borrow().desk.saver.slot().index()),
        )
    }

    pub(super) fn spawn_window(&mut self, slot: usize, spec: OpenSpec) -> Result<(), String> {
        self.next_key = self.next_key.wrapping_add(1);
        let key = WindowKey(self.next_key);
        let shared = self
            .windows
            .first()
            .map(|w| w.state.borrow().desk.preferences.clone());
        let slot_obj = self.role.slot(slot);
        let mut handle = (self.builder)(key, slot_obj, shared, spec, &self.update)?;
        self.offer.opened(&mut handle);
        self.windows.push(handle);
        Ok(())
    }
}
