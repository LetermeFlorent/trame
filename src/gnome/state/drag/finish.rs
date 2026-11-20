use trame::geometry::{Point, Rect};
use trame::session::SessionId;
use trame::ui::drop::{DragView, DropSpot};
use trame::ui::{Hit, SidebarItem};

use super::{Drag, Lifted};
use crate::gnome::state::WindowState;

impl WindowState {
    pub(super) fn instance_drag_moved(&mut self, point: Point, threshold: f32) -> Option<bool> {
        let (id, start) = match self.drag {
            Some(Drag::PendingInstance(id, origin)) => {
                if origin.distance_to(point) < threshold {
                    return Some(false);
                }
                self.drop_press();
                (id, self.desk.workspace.instance_index(id)?)
            }
            Some(Drag::Instance(id, start, _)) => (id, start),
            _ => return None,
        };
        self.drag = Some(Drag::Instance(id, start, point));
        let to = self.instance_target(point);
        if let (Some(from), Some(to)) = (self.desk.workspace.instance_index(id), to)
            && self.desk.workspace.move_instance(from, to)
        {
            self.changed();
        }
        self.dirty = true;
        Some(true)
    }

    pub fn instance_target(&self, point: Point) -> Option<usize> {
        match self.frame.hit(point, false)? {
            Hit::Row(SidebarItem::Instance(id)) | Hit::RailInstance(id) => {
                self.desk.workspace.instance_index(id)
            }
            Hit::Row(SidebarItem::Tab(tab)) => Some(self.desk.workspace.locate(tab)?.instance),
            _ => None,
        }
    }

    pub fn drag_released(&mut self) -> bool {
        let outside = |p: Point, bounds: Rect| !bounds.contains(p);
        match self.drag.take() {
            Some(Drag::Tab(tab, p)) => {
                if outside(p, self.frame.bounds) && self.desk.can_tear(tab) {
                    self.drag = Some(Drag::Out(Lifted::Tab(tab)));
                } else if let Some(spot) = self.drop_spot(tab, p) {
                    self.desk.workspace.move_tab(tab, spot.target);
                    self.motion.animate_next();
                }
                true
            }
            Some(Drag::Instance(id, _, p)) => {
                self.desk.workspace.renumber_after_move();
                if outside(p, self.frame.bounds) && self.desk.workspace.instances.len() > 1 {
                    self.drag = Some(Drag::Out(Lifted::Instance(id)));
                }
                true
            }
            Some(Drag::Divider(_) | Drag::Sidebar(_) | Drag::List(..) | Drag::Picker(_)) => true,
            _ => false,
        }
    }

    pub fn cancel_drag(&mut self) -> bool {
        let Some(drag) = self.drag.take() else {
            return false;
        };
        if matches!(drag, Drag::Pending(..) | Drag::PendingInstance(..)) {
            return false;
        }
        self.restore_cancelled_drag(drag);
        self.drop_press();
        self.changed();
        true
    }

    fn restore_cancelled_drag(&mut self, drag: Drag) {
        match drag {
            Drag::Divider(drag) => {
                if let Some(inst) = self.desk.workspace.active_instance_mut() {
                    inst.root = drag.root;
                }
            }
            Drag::Sidebar(drag) => {
                self.desk.workspace.sidebar_width = drag.width;
                self.desk.workspace.sidebar = drag.mode;
            }
            Drag::Instance(id, start, _) => {
                if let Some(from) = self.desk.workspace.instance_index(id) {
                    self.desk.workspace.move_instance(from, start);
                }
            }
            _ => {}
        }
    }

    pub fn drag_view(&self) -> Option<DragView> {
        let Some(Drag::Tab(tab, pointer)) = self.drag else {
            return None;
        };
        Some(DragView {
            label: self.tab_title(tab),
            pointer,
            mark: self.drop_spot(tab, pointer).map(|s| s.mark),
            ghost: true,
        })
    }

    fn drop_spot(&self, tab: SessionId, pointer: Point) -> Option<DropSpot> {
        let instance = self.desk.workspace.active_instance()?.id;
        let spot = self.frame.drop_at(pointer, instance)?;
        self.desk
            .workspace
            .clone()
            .move_tab(tab, spot.target)
            .then_some(spot)
    }
}
