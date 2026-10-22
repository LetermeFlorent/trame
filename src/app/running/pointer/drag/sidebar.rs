use trame::geometry::Point;
use trame::ui::frame::dragged_width;

use crate::app::running::Running;

#[derive(Clone, Copy)]
pub(in crate::app::running) struct SidebarDrag {
    width: Option<f32>,
    open: bool,
}

impl Running {
    pub(super) fn sidebar_drag(&self) -> SidebarDrag {
        SidebarDrag {
            width: self.workspace.sidebar_width,
            open: self.workspace.sidebar_open,
        }
    }

    pub(super) fn drag_sidebar(&mut self, drag: SidebarDrag, point: Point) {
        let pointer = (point.x - self.frame.side.x) / self.metrics.scale;
        let width = dragged_width(pointer);
        let open = width.is_some();
        if open != self.workspace.sidebar_open {
            self.workspace.sidebar_open = open;
            self.motion.animate_next();
        }
        self.workspace.sidebar_width = width.or(drag.width);
    }

    pub(super) fn restore_sidebar(&mut self, drag: SidebarDrag) {
        if drag.open != self.workspace.sidebar_open {
            self.motion.animate_next();
        }
        self.workspace.sidebar_width = drag.width;
        self.workspace.sidebar_open = drag.open;
    }
}
