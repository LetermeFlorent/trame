use std::error::Error;
use std::mem;

use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use trame::log_error;
use trame::workspace::Snapshot;

use super::App;
use super::persist::free_slot;
use super::running::{Quit, Running};

impl App {
    pub(super) fn open_saved(&mut self, event_loop: &ActiveEventLoop) {
        let mut failure = None;
        for saved in mem::take(&mut self.saved) {
            if let Err(error) = self.launch(event_loop, saved.slot, saved.snapshot) {
                log_error!("a saved window could not open: {error}");
                failure = Some(error);
            }
        }
        if self.windows.is_empty() {
            self.failure = failure;
            event_loop.exit();
        }
    }

    pub(super) fn open_another(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            return;
        }
        let slot = free_slot(self.windows.iter().map(Running::slot));
        match self.launch(event_loop, slot, None) {
            Ok(()) => self
                .windows
                .iter()
                .last()
                .map_or((), Running::bring_forward),
            Err(error) => log_error!("a new window could not open: {error}"),
        }
    }

    pub(super) fn on(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        act: impl FnOnce(&mut Running),
    ) {
        if let Some(running) = self.windows.iter_mut().find(|running| running.id() == id) {
            act(running);
        }
        self.share_preferences(id);
        self.settle(event_loop);
    }

    pub(super) fn settle(&mut self, event_loop: &ActiveEventLoop) {
        let quits: Vec<(WindowId, Quit)> = self
            .windows
            .iter_mut()
            .filter_map(|running| running.take_quit().map(|quit| (running.id(), quit)))
            .collect();
        for (id, quit) in quits {
            match quit {
                Quit::Close => self.close_window(event_loop, id),
                Quit::Restart(restart) => self.restart_for(event_loop, restart),
            }
        }
        self.process.follow(self.windows.iter().all(Running::rests));
    }

    fn launch(
        &mut self,
        event_loop: &ActiveEventLoop,
        slot: usize,
        saved: Option<Snapshot>,
    ) -> Result<(), Box<dyn Error>> {
        let shared = self
            .windows
            .first()
            .map(|running| running.preferences().clone());
        let slot = self.role.slot(slot);
        let opened = Running::open_window(event_loop, self.proxy.clone(), saved, slot, shared)?;
        let mut running = opened.show(&self.update)?;
        self.offer.opened(&mut running);
        self.windows.push(running);
        Ok(())
    }

    fn close_window(&mut self, event_loop: &ActiveEventLoop, id: WindowId) {
        if self.windows.len() <= 1 {
            return event_loop.exit();
        }
        let Some(index) = self.windows.iter().position(|running| running.id() == id) else {
            return;
        };
        let mut running = self.windows.remove(index);
        running.retire();
        if let Err(error) = self.role.slot(running.slot()).forget() {
            log_error!("a closed window's files could not be removed: {error}");
        }
    }

    fn share_preferences(&mut self, id: WindowId) {
        let Some(source) = self.windows.iter().find(|running| running.id() == id) else {
            return;
        };
        let preferences = source.preferences();
        if self
            .windows
            .iter()
            .all(|running| running.preferences() == preferences)
        {
            return;
        }
        let preferences = preferences.clone();
        for running in &mut self.windows {
            running.follow_preferences(&preferences);
        }
    }
}
