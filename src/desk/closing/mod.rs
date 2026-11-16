mod question;

use std::rc::Rc;

use crate::host::quit::Quit;
use crate::preferences::SshHost;
use crate::workspace::{Instance, InstanceId};

use super::{Desk, Effect, Effects, PendingPaste};

pub use question::{Confirm, Question};

#[derive(Clone, Debug)]
pub enum Closing {
    Window,
    Instance(InstanceId),
    Update,
    Forget(SshHost),
    Paste(Rc<PendingPaste>),
}

impl Desk {
    pub fn ask_close(&mut self, closing: Closing, loading: bool) -> Effects {
        let asks = match closing {
            Closing::Update => true,
            _ => self
                .preferences
                .confirms_closing(self.tabs_closing(&closing)),
        };
        if asks && !loading {
            self.closing = Some(closing);
            return Effect::Changed.into();
        }
        self.close(closing)
    }

    pub fn settle_closing(&mut self, confirmed: bool) -> Effects {
        let closing = self.closing.take();
        let mut effects = match closing.filter(|_| confirmed) {
            Some(closing) => self.close(closing),
            None => Effects::default(),
        };
        effects.push(Effect::Changed);
        effects
    }

    pub fn tabs_closing(&self, closing: &Closing) -> usize {
        let instances = &self.workspace.instances;
        match closing {
            Closing::Window => instances.iter().map(Instance::tab_count).sum(),
            Closing::Instance(id) => instances
                .iter()
                .find(|instance| instance.id == *id)
                .map_or(0, Instance::tab_count),
            Closing::Update | Closing::Forget(_) | Closing::Paste(_) => 0,
        }
    }

    fn close(&mut self, closing: Closing) -> Effects {
        match closing {
            Closing::Window => self.quit_with(Quit::Close),
            Closing::Instance(id) => return self.close_instance(id),
            Closing::Update => self.restart_for_update(),
            Closing::Forget(host) => {
                let mut effects = self.forget_host(&host);
                effects.push(Effect::HostsChanged);
                return effects;
            }
            Closing::Paste(pending) => return self.send_paste(&pending),
        }
        Effects::default()
    }
}
