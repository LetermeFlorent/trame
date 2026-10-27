use winit::keyboard::{Key, NamedKey};

use crate::app::running::Running;

use super::{Drag, Ghost};

impl Running {
    pub(in crate::app::running) fn drag_key(&mut self, key: &Key) -> bool {
        *key == Key::Named(NamedKey::Escape) && self.cancel_drag()
    }

    pub(in crate::app::running) fn cancel_drag(&mut self) -> bool {
        let drag = match self.drag.take() {
            None | Some(Drag::Pending(..) | Drag::PendingInstance(..)) => return false,
            Some(drag) => drag,
        };
        match drag {
            Drag::Divider(drag) => {
                if let Some(instance) = self.workspace.active_instance_mut() {
                    instance.root = drag.root;
                }
            }
            Drag::Sidebar(drag) => self.restore_sidebar(drag),
            Drag::Instance(id, start) => self.move_instance_to(id, Some(start)),
            _ => {}
        }
        self.drop_press();
        self.pointer.ghost = Ghost::Idle;
        self.changed();
        true
    }
}
