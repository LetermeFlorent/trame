mod cancel;
mod ghost;
mod instance;
mod sidebar;
mod tab;

use winit::window::{Cursor, CursorIcon};

use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::Hit;
use trame::ui::frame::{BarPart, ListControl, PickerPart, SidebarItem};
use trame::workspace::{Axis, Divider, InstanceId, Node, resized};

use crate::app::running::Running;
use crate::app::running::settings::ListKind;
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
    List(ListKind, f32),
    Picker(PickerPart),
    PendingInstance(InstanceId, Point),
    Instance(InstanceId, usize),
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
            Hit::List(ListControl::Thumb) => self.list_drag(ListKind::Hosts, origin),
            Hit::SidebarBar(BarPart::Thumb) => self.list_drag(ListKind::Sidebar, origin),
            Hit::Picker(part) => Some(Drag::Picker(part)),
            Hit::Row(SidebarItem::Instance(id)) | Hit::RailInstance(id) => {
                Some(Drag::PendingInstance(id, origin))
            }
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
        if let Some(moved) = self.instance_drag_moved(point, threshold) {
            return moved;
        }
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
                self.drag_divider(&drag, point);
                Drag::Divider(drag)
            }
            Some(Drag::Sidebar(drag)) => {
                self.drag_sidebar(drag, point);
                Drag::Sidebar(drag)
            }
            Some(Drag::List(list, grab)) => {
                self.drag_list(list, point.y - grab);
                Drag::List(list, grab)
            }
            Some(Drag::Picker(part)) => {
                self.drag_picker(part);
                Drag::Picker(part)
            }
            Some(drag @ (Drag::PendingInstance(..) | Drag::Instance(..))) => drag,
        };
        self.drag = Some(next);
        self.window.request_redraw();
        true
    }

    fn drag_divider(&mut self, drag: &DividerDrag, point: Point) {
        let delta = drag.divider.delta(drag.origin, point);
        if let Some(instance) = self.workspace.active_instance_mut() {
            instance.root = resized(&drag.root, &drag.divider, delta);
        }
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
            Some(Drag::Instance(..)) => Some(Cursor::Icon(CursorIcon::Grabbing)),
            _ => None,
        }
    }
}

fn distance(from: Point, to: Point) -> f32 {
    (to.x - from.x).hypot(to.y - from.y)
}
