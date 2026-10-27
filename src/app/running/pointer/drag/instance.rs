use trame::geometry::Point;
use trame::ui::Hit;
use trame::ui::frame::SidebarItem;
use trame::workspace::InstanceId;

use super::{Drag, distance};
use crate::app::running::Running;

impl Running {
    pub(super) fn instance_drag_moved(&mut self, point: Point, threshold: f32) -> Option<bool> {
        let (id, start) = match self.drag {
            Some(Drag::PendingInstance(id, origin)) => {
                if distance(origin, point) < threshold {
                    return Some(false);
                }
                self.drop_press();
                (id, self.workspace.instance_index(id)?)
            }
            Some(Drag::Instance(id, start)) => (id, start),
            _ => return None,
        };
        self.drag = Some(Drag::Instance(id, start));
        self.move_instance_to(id, self.instance_target(point));
        self.window.request_redraw();
        Some(true)
    }

    pub(super) fn move_instance_to(&mut self, id: InstanceId, target: Option<usize>) {
        let from = self.workspace.instance_index(id);
        let moved = match (from, target) {
            (Some(from), Some(to)) => self.workspace.move_instance(from, to),
            _ => false,
        };
        if moved {
            self.changed();
        }
    }

    fn instance_target(&self, point: Point) -> Option<usize> {
        match self.frame.hit(point, false)? {
            Hit::Row(SidebarItem::Instance(id)) | Hit::RailInstance(id) => {
                self.workspace.instance_index(id)
            }
            Hit::Row(SidebarItem::Tab(tab)) => Some(self.workspace.locate(tab)?.instance),
            _ => None,
        }
    }
}
