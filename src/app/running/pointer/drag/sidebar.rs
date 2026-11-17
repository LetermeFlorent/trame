use trame::geometry::Point;
use trame::ui::frame::dragged_width;
use trame::ui::{PanelSide, SidebarMode};

use crate::app::running::Running;

#[derive(Clone, Copy)]
pub(in crate::app::running) struct SidebarDrag {
    width: Option<f32>,
    mode: SidebarMode,
}

impl Running {
    pub(super) fn sidebar_drag(&self) -> SidebarDrag {
        SidebarDrag {
            width: self.desk.workspace.sidebar_width,
            mode: self.desk.workspace.sidebar,
        }
    }

    pub(super) fn drag_sidebar(&mut self, drag: SidebarDrag, point: Point) {
        let side = self.frame.side;
        let reach = match self.metrics.panel {
            PanelSide::Left => point.x - side.x,
            PanelSide::Right => side.right() - point.x,
        };
        let pointer = reach / self.metrics.scale;
        let width = dragged_width(pointer);
        let mode = match width {
            Some(_) => SidebarMode::Open,
            None => SidebarMode::Rail,
        };
        if mode != self.desk.workspace.sidebar {
            self.desk.workspace.sidebar = mode;
            self.motion.animate_next();
        }
        self.desk.workspace.sidebar_width = width.or(drag.width);
    }

    pub(super) fn restore_sidebar(&mut self, drag: SidebarDrag) {
        if drag.mode != self.desk.workspace.sidebar {
            self.motion.animate_next();
        }
        self.desk.workspace.sidebar_width = drag.width;
        self.desk.workspace.sidebar = drag.mode;
    }
}
