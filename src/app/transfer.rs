use winit::dpi::PhysicalPosition;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use trame::host::persist::free_slot;
use trame::log_error;
use trame::session::SessionId;

use super::App;
use super::running::{Lifted, Running};

impl App {
    pub(super) fn drop_out(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        lifted: Lifted,
        at: PhysicalPosition<i32>,
    ) {
        let Some(source) = self.windows.iter().position(|running| running.id() == id) else {
            return;
        };
        let target = self
            .windows
            .iter()
            .position(|running| running.id() != id && running.holds(at));
        match (lifted, target) {
            (Lifted::Tab(tab), None) => self.tear_off(event_loop, id, tab, at),
            (Lifted::Tab(tab), Some(target)) => {
                if let Some(torn) = self.windows[source].tear_tab(tab) {
                    self.windows[target].take_dropped_tab(torn, at);
                }
            }
            (Lifted::Instance(instance), Some(target)) => {
                if let Some(moved) = self.windows[source].give_instance(instance) {
                    self.windows[target].take_moved_instance(moved, at);
                }
            }
            (Lifted::Instance(_), None) => {}
        }
    }

    fn tear_off(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        tab: SessionId,
        at: PhysicalPosition<i32>,
    ) {
        let Some(source) = self.windows.iter().position(|running| running.id() == id) else {
            return;
        };
        if !self.windows[source].can_tear(tab) {
            return;
        }
        let shared = Some(self.windows[source].preferences().clone());
        let slot = free_slot(self.windows.iter().map(Running::slot));
        let slot = self.role.slot(slot);
        let opened = match Running::open_window(event_loop, self.proxy.clone(), None, slot, shared)
        {
            Ok(opened) => opened,
            Err(error) => return log_error!("a torn tab could not open its window: {error}"),
        };
        let Some(torn) = self.windows[source].tear_tab(tab) else {
            return;
        };
        let mut running = opened.adopt(&self.update, torn, at);
        self.offer.opened(&mut running);
        running.bring_forward();
        self.windows.push(running);
    }
}
