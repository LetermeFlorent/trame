use winit::dpi::PhysicalPosition;

use trame::geometry::{Point, Rect};
use trame::session::SessionId;
use trame::ui::drop::{DragView, DropSpot};

use super::{Drag, Ghost};
use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn drag_released(&mut self) -> bool {
        self.pointer.ghost = Ghost::Idle;
        match self.drag.take() {
            Some(Drag::Tab(tab, pointer)) => {
                if let Some(at) = self.torn_at(tab, pointer) {
                    self.drag = Some(Drag::Out(tab, at));
                } else if let Some(spot) = self.drop_spot(tab, pointer) {
                    self.workspace.move_tab(tab, spot.target);
                    self.motion.animate_next();
                }
                true
            }
            Some(Drag::Instance(..)) => {
                self.workspace.renumber_after_move();
                true
            }
            Some(Drag::Divider(_) | Drag::Sidebar(_) | Drag::List(..) | Drag::Picker(_)) => true,
            _ => false,
        }
    }

    pub(in crate::app::running) fn drag_view(&self) -> Option<DragView> {
        let Some(Drag::Tab(tab, pointer)) = self.drag else {
            return None;
        };
        Some(DragView {
            label: self.tab_title(tab),
            pointer,
            mark: self.drop_spot(tab, pointer).map(|spot| spot.mark),
            ghost: self.pointer.ghost.painted(),
        })
    }

    pub fn take_torn(&mut self) -> Option<(SessionId, PhysicalPosition<i32>)> {
        let Some(Drag::Out(tab, at)) = self.drag else {
            return None;
        };
        self.drag = None;
        Some((tab, at))
    }

    fn torn_at(&self, tab: SessionId, pointer: Point) -> Option<PhysicalPosition<i32>> {
        let size = self.window.inner_size();
        let window = Rect::new(0.0, 0.0, size.width as f32, size.height as f32);
        if window.contains(pointer) || !self.can_tear(tab) {
            return None;
        }
        let origin = self.window.inner_position().ok()?;
        let x = origin.x + pointer.x.round() as i32;
        let y = origin.y + pointer.y.round() as i32;
        Some(PhysicalPosition::new(x, y))
    }

    fn drop_spot(&self, tab: SessionId, pointer: Point) -> Option<DropSpot> {
        let instance = self.workspace.active_instance()?.id;
        let spot = self.frame.drop_at(pointer, instance)?;
        self.workspace
            .clone()
            .move_tab(tab, spot.target)
            .then_some(spot)
    }
}
