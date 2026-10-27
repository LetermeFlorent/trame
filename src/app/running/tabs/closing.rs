use winit::event_loop::ActiveEventLoop;

use trame::workspace::{Instance, InstanceId};

use crate::app::running::Running;

#[derive(Clone, Copy)]
pub(in crate::app::running) enum Closing {
    Window,
    Instance(InstanceId),
    Update,
}

impl Running {
    pub(in crate::app::running) fn ask_close(
        &mut self,
        event_loop: &ActiveEventLoop,
        closing: Closing,
    ) {
        let asks = match closing {
            Closing::Update => true,
            _ => self
                .preferences
                .confirms_closing(self.tabs_closing(closing)),
        };
        if asks && self.loader.is_none() {
            self.closing = Some(closing);
            return self.changed();
        }
        self.close(event_loop, closing);
    }

    fn close(&mut self, event_loop: &ActiveEventLoop, closing: Closing) {
        match closing {
            Closing::Window => event_loop.exit(),
            Closing::Instance(id) => self.close_instance(event_loop, id),
            Closing::Update => self.restart_for_update(event_loop),
        }
    }

    fn close_instance(&mut self, event_loop: &ActiveEventLoop, instance: InstanceId) {
        let Some(index) = self.workspace.instance_index(instance) else {
            return;
        };
        for id in self.workspace.remove_instance(index) {
            self.sessions.remove(id);
        }
        self.after_close(event_loop);
    }

    pub(super) fn tabs_closing(&self, closing: Closing) -> usize {
        let instances = &self.workspace.instances;
        match closing {
            Closing::Window => instances.iter().map(Instance::tab_count).sum(),
            Closing::Instance(id) => instances
                .iter()
                .find(|instance| instance.id == id)
                .map_or(0, Instance::tab_count),
            Closing::Update => 0,
        }
    }

    pub(super) fn settle_closing(&mut self, event_loop: &ActiveEventLoop, confirmed: bool) {
        let closing = self.closing.take();
        if let Some(closing) = closing.filter(|_| confirmed) {
            self.close(event_loop, closing);
        }
        self.changed();
    }
}
