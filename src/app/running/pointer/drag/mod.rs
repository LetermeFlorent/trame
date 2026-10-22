mod ghost;
mod sidebar;
mod tab;

use winit::keyboard::{Key, NamedKey};
use winit::window::{Cursor, CursorIcon};

use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::Hit;
use trame::workspace::{Axis, Divider, Node, resized};

use crate::app::running::Running;
pub(in crate::app::running) use ghost::Ghost;
use sidebar::SidebarDrag;

const DRAG_THRESHOLD: f32 = 4.0;

pub(in crate::app::running) struct DividerDrag {
    divider: Divider,
    origin: Point,
    root: Node,
}

pub(in crate::app::running) enum Drag {
    Pending(SessionId, Point),
    Tab(SessionId, Point),
    Divider(DividerDrag),
    Sidebar(SidebarDrag),
}

impl Running {
    pub(in crate::app::running) fn drag_pressed(&mut self, hit: Hit) {
        let Some(origin) = self.pointer.position else {
            return;
        };
        self.drag = match hit {
            Hit::Tab(_, tab) => Some(Drag::Pending(tab, origin)),
            Hit::Divider(index) => self.divider_drag(index, origin).map(Drag::Divider),
            Hit::SidebarEdge => Some(Drag::Sidebar(self.sidebar_drag())),
            _ => None,
        };
    }

    fn divider_drag(&self, index: usize, origin: Point) -> Option<DividerDrag> {
        let divider = self.frame.dividers.get(index)?.clone();
        let root = self.workspace.active_instance()?.root.clone();
        Some(DividerDrag {
            divider,
            origin,
            root,
        })
    }

    pub(in crate::app::running) fn drag_moved(&mut self, point: Point) -> bool {
        let threshold = self.metrics.px(DRAG_THRESHOLD);
        let next = match self.drag.take() {
            None => return false,
            Some(Drag::Pending(tab, origin)) if distance(origin, point) < threshold => {
                self.drag = Some(Drag::Pending(tab, origin));
                return false;
            }
            Some(Drag::Pending(tab, _) | Drag::Tab(tab, _)) => {
                self.view.pressed = None;
                Drag::Tab(tab, point)
            }
            Some(Drag::Divider(drag)) => {
                let delta = drag.divider.delta(drag.origin, point);
                if let Some(instance) = self.workspace.active_instance_mut() {
                    instance.root = resized(&drag.root, &drag.divider, delta);
                }
                Drag::Divider(drag)
            }
            Some(Drag::Sidebar(drag)) => {
                self.drag_sidebar(drag, point);
                Drag::Sidebar(drag)
            }
        };
        self.drag = Some(next);
        self.window.request_redraw();
        true
    }

    pub(in crate::app::running) fn drag_key(&mut self, key: &Key) -> bool {
        *key == Key::Named(NamedKey::Escape) && self.cancel_drag()
    }

    pub(in crate::app::running) fn cancel_drag(&mut self) -> bool {
        let drag = match self.drag.take() {
            None | Some(Drag::Pending(..)) => return false,
            Some(drag) => drag,
        };
        match drag {
            Drag::Divider(drag) => {
                if let Some(instance) = self.workspace.active_instance_mut() {
                    instance.root = drag.root;
                }
            }
            Drag::Sidebar(drag) => self.restore_sidebar(drag),
            _ => {}
        }
        self.pointer.pressed = None;
        self.pointer.ghost = Ghost::Idle;
        self.view.pressed = None;
        self.changed();
        true
    }

    pub(in crate::app::running) fn drags_tab(&self) -> bool {
        matches!(self.drag, Some(Drag::Tab(..)))
    }

    pub(in crate::app::running) fn drag_cursor(&self) -> Option<Cursor> {
        match &self.drag {
            Some(Drag::Divider(drag)) => Some(Cursor::Icon(match drag.divider.axis {
                Axis::Row => CursorIcon::ColResize,
                Axis::Column => CursorIcon::RowResize,
            })),
            Some(Drag::Sidebar(_)) => Some(Cursor::Icon(CursorIcon::ColResize)),
            Some(Drag::Tab(..)) => Some(self.pointer.ghost.cursor()),
            _ => None,
        }
    }
}

fn distance(from: Point, to: Point) -> f32 {
    (to.x - from.x).hypot(to.y - from.y)
}
