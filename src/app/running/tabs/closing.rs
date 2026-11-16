use std::rc::Rc;

use trame::host::quit::Quit;
use trame::workspace::{Instance, InstanceId};

use crate::app::running::Running;
use crate::app::running::keys::PendingPaste;

#[derive(Clone)]
pub(in crate::app::running) enum Closing {
    Window,
    Instance(InstanceId),
    Update,
    Forget(usize),
    Paste(Rc<PendingPaste>),
}

impl Running {
    pub(in crate::app::running) fn ask_close(&mut self, closing: Closing) {
        let asks = match closing {
            Closing::Update => true,
            _ => self
                .desk
                .preferences
                .confirms_closing(self.tabs_closing(&closing)),
        };
        if asks && self.loader.is_none() {
            self.closing = Some(closing);
            return self.changed();
        }
        self.close(closing);
    }

    fn close(&mut self, closing: Closing) {
        match closing {
            Closing::Window => self.desk.quit_with(Quit::Close),
            Closing::Instance(id) => self.close_instance(id),
            Closing::Update => self.desk.restart_for_update(),
            Closing::Forget(index) => self.forget_at(index),
            Closing::Paste(pending) => self.send_paste(&pending),
        }
    }

    fn close_instance(&mut self, instance: InstanceId) {
        let Some(index) = self.desk.workspace.instance_index(instance) else {
            return;
        };
        for id in self.desk.workspace.remove_instance(index) {
            self.desk.sessions.remove(id);
        }
        self.after_close();
    }

    pub(super) fn tabs_closing(&self, closing: &Closing) -> usize {
        let instances = &self.desk.workspace.instances;
        match closing {
            Closing::Window => instances.iter().map(Instance::tab_count).sum(),
            Closing::Instance(id) => instances
                .iter()
                .find(|instance| instance.id == *id)
                .map_or(0, Instance::tab_count),
            Closing::Update | Closing::Forget(_) | Closing::Paste(_) => 0,
        }
    }

    pub(super) fn settle_closing(&mut self, confirmed: bool) {
        let closing = self.closing.take();
        if let Some(closing) = closing.filter(|_| confirmed) {
            self.close(closing);
        }
        self.changed();
    }
}
