mod finish;

use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::frame::SidebarItem;
use trame::ui::frame::{BarPart, ListControl, PickerPart, dragged_width};
use trame::ui::{Hit, PanelSide, SidebarMode};
use trame::workspace::{Axis, Divider, InstanceId, Node, resized};

use super::WindowState;
use super::settings::ListKind;

const DRAG_THRESHOLD: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifted {
    Tab(SessionId),
    Instance(InstanceId),
}

#[derive(Clone, Copy)]
pub struct SidebarDrag {
    pub(super) width: Option<f32>,
    pub(super) mode: SidebarMode,
}

pub struct DividerDrag {
    pub axis: Axis,
    divider: Divider,
    origin: Point,
    pub(super) root: Node,
}

pub enum Drag {
    Pending(SessionId, Point),
    Tab(SessionId, Point),
    Out(Lifted),
    Divider(DividerDrag),
    Sidebar(SidebarDrag),
    List(ListKind, f32),
    Picker(PickerPart),
    PendingInstance(InstanceId, Point),
    Instance(InstanceId, usize, Point),
}

impl WindowState {
    pub fn drag_pressed(&mut self, hit: Hit) {
        let Some(origin) = self.pointer else {
            return;
        };
        self.drag = match hit {
            Hit::Tab(_, tab) | Hit::Row(SidebarItem::Tab(tab)) => Some(Drag::Pending(tab, origin)),
            Hit::Divider(i) => self.divider_drag(i, origin).map(Drag::Divider),
            Hit::SidebarEdge if self.desk.workspace.sidebar != SidebarMode::Hidden => {
                Some(Drag::Sidebar(SidebarDrag {
                    width: self.desk.workspace.sidebar_width,
                    mode: self.desk.workspace.sidebar,
                }))
            }
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
        let root = self.desk.workspace.active_instance()?.root.clone();
        Some(DividerDrag {
            axis: divider.axis,
            divider,
            origin,
            root,
        })
    }

    pub fn drag_moved(&mut self, point: Point) -> bool {
        let threshold = self.metrics.px(DRAG_THRESHOLD);
        if let Some(moved) = self.instance_drag_moved(point, threshold) {
            return moved;
        }
        let Some(current) = self.drag.take() else {
            return false;
        };
        let (next, moved) = self.step_drag(current, point, threshold);
        self.drag = Some(next);
        if moved {
            self.changed();
        }
        moved
    }

    fn step_drag(&mut self, current: Drag, point: Point, threshold: f32) -> (Drag, bool) {
        let next = match current {
            Drag::Pending(tab, origin) if origin.distance_to(point) < threshold => {
                return (Drag::Pending(tab, origin), false);
            }
            Drag::Pending(tab, _) | Drag::Tab(tab, _) => {
                self.view.pressed = None;
                Drag::Tab(tab, point)
            }
            Drag::Divider(drag) => {
                let delta = drag.divider.delta(drag.origin, point);
                if let Some(inst) = self.desk.workspace.active_instance_mut() {
                    inst.root = resized(&drag.root, &drag.divider, delta);
                }
                Drag::Divider(drag)
            }
            Drag::Sidebar(drag) => {
                self.drag_sidebar(drag, point);
                Drag::Sidebar(drag)
            }
            Drag::List(list, grab) => {
                self.drag_list(list, point.y - grab);
                Drag::List(list, grab)
            }
            Drag::Picker(part) => {
                self.drag_picker(part);
                Drag::Picker(part)
            }
            other => other,
        };
        (next, true)
    }

    fn drag_sidebar(&mut self, drag: SidebarDrag, point: Point) {
        let side = self.frame.side;
        let reach = match self.metrics.panel {
            PanelSide::Left => point.x - side.x,
            PanelSide::Right => side.right() - point.x,
        };
        let width = dragged_width(reach / self.metrics.scale);
        let mode = if width.is_some() {
            SidebarMode::Open
        } else {
            SidebarMode::Rail
        };
        if mode != self.desk.workspace.sidebar {
            self.desk.workspace.sidebar = mode;
            self.motion.animate_next();
        }
        self.desk.workspace.sidebar_width = width.or(drag.width);
    }
}
