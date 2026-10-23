use winit::event::{ElementState, KeyEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};

use trame::ui::Hit;
use trame::ui::frame::{ConfirmRequest, FormAction};
use trame::workspace::{Instance, InstanceId};

use crate::app::running::Running;
use crate::app::running::keys::is_escape;

#[derive(Clone, Copy)]
pub(in crate::app::running) enum Closing {
    Window,
    Instance(InstanceId),
}

impl Running {
    pub(in crate::app::running) fn ask_close(
        &mut self,
        event_loop: &ActiveEventLoop,
        closing: Closing,
    ) {
        let asks = self
            .preferences
            .confirms_closing(self.tabs_closing(closing));
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

    fn tabs_closing(&self, closing: Closing) -> usize {
        let instances = &self.workspace.instances;
        match closing {
            Closing::Window => instances.iter().map(Instance::tab_count).sum(),
            Closing::Instance(id) => instances
                .iter()
                .find(|instance| instance.id == id)
                .map_or(0, Instance::tab_count),
        }
    }

    fn settle_closing(&mut self, event_loop: &ActiveEventLoop, confirmed: bool) {
        let closing = self.closing.take();
        if let Some(closing) = closing.filter(|_| confirmed) {
            self.close(event_loop, closing);
        }
        self.changed();
    }

    pub(in crate::app::running) fn place_confirm(&mut self) {
        let Some(closing) = self.closing else {
            return;
        };
        let count = self.tabs_closing(closing);
        if count == 0 {
            self.closing = None;
            return;
        }
        let strings = self.strings;
        let title = match closing {
            Closing::Window => strings.close_window,
            Closing::Instance(_) => strings.close_instance,
        };
        let message = format!("{count} {}", strings.tabs_will_close);
        let request = ConfirmRequest {
            title,
            message: &message,
            strings,
        };
        self.frame.place_confirm(&mut self.text, &request);
    }

    pub(in crate::app::running) fn closing_button(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
    ) -> bool {
        if self.closing.is_none() {
            return false;
        }
        let hit = self.hit_under_pointer();
        if !matches!(
            hit,
            Some(Hit::FormButton(_) | Hit::EditClose | Hit::Dialog | Hit::Backdrop)
        ) {
            return false;
        }
        if state == ElementState::Pressed && hit != Some(Hit::Dialog) {
            let confirmed = hit == Some(Hit::FormButton(FormAction::Close));
            self.settle_closing(event_loop, confirmed);
        }
        true
    }

    pub(in crate::app::running) fn closing_key(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &KeyEvent,
    ) -> bool {
        if self.closing.is_none() {
            return false;
        }
        let confirmed = event.logical_key == Key::Named(NamedKey::Enter);
        if confirmed || is_escape(event) {
            self.settle_closing(event_loop, confirmed);
        }
        true
    }
}
