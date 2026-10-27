use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::drop::{DragView, DropSpot};

use super::{Drag, Ghost};
use crate::app::running::Running;

impl Running {
    pub(in crate::app::running) fn drag_released(&mut self) -> bool {
        self.pointer.ghost = Ghost::Idle;
        match self.drag.take() {
            Some(Drag::Tab(tab, pointer)) => {
                if let Some(spot) = self.drop_spot(tab, pointer) {
                    self.workspace.move_tab(tab, spot.target);
                    self.motion.animate_next();
                }
                true
            }
            Some(Drag::Instance(..)) => {
                self.workspace.renumber_after_move();
                true
            }
            Some(Drag::Divider(_) | Drag::Sidebar(_) | Drag::List(_) | Drag::Picker(_)) => true,
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

    fn drop_spot(&self, tab: SessionId, pointer: Point) -> Option<DropSpot> {
        let instance = self.workspace.active_instance()?.id;
        let spot = self.frame.drop_at(pointer, instance)?;
        self.workspace
            .clone()
            .move_tab(tab, spot.target)
            .then_some(spot)
    }
}
