use winit::dpi::PhysicalPosition;

use trame::geometry::{Point, Rect};
use trame::session::SessionId;
use trame::ui::drop::{DragView, DropSpot};

use super::{Drag, Ghost};
use crate::app::running::{Lifted, Running};

impl Running {
    pub(in crate::app::running) fn drag_released(&mut self) -> bool {
        self.pointer.ghost = Ghost::Idle;
        match self.drag.take() {
            Some(Drag::Tab(tab, pointer)) => {
                if let Some(at) = self.outside_at(pointer) {
                    self.drag = Some(Drag::Out(Lifted::Tab(tab), at));
                } else if let Some(spot) = self.drop_spot(tab, pointer) {
                    self.workspace.move_tab(tab, spot.target);
                    self.motion.animate_next();
                }
                true
            }
            Some(Drag::Instance(id, _, pointer)) => {
                self.workspace.renumber_after_move();
                if let Some(at) = self.outside_at(pointer) {
                    self.drag = Some(Drag::Out(Lifted::Instance(id), at));
                }
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

    pub fn take_out(&mut self) -> Option<(Lifted, PhysicalPosition<i32>)> {
        let Some(Drag::Out(lifted, at)) = self.drag else {
            return None;
        };
        self.drag = None;
        Some((lifted, at))
    }

    fn outside_at(&self, pointer: Point) -> Option<PhysicalPosition<i32>> {
        let size = self.window.inner_size();
        let window = Rect::new(0.0, 0.0, size.width as f32, size.height as f32);
        if window.contains(pointer) {
            return None;
        }
        self.screen_point(pointer)
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
