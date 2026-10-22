use winit::keyboard::{Key, NamedKey};
use winit::window::CursorIcon;

use trame::geometry::Point;
use trame::session::SessionId;
use trame::ui::Hit;
use trame::ui::drop::{DragView, DropSpot};
use trame::workspace::{Axis, Divider, DropTarget, Node, resized};

use super::Running;

const DRAG_THRESHOLD: f32 = 4.0;

pub(super) struct DividerDrag {
    divider: Divider,
    origin: Point,
    root: Node,
}

pub(super) enum Drag {
    Pending(SessionId, Point),
    Tab(SessionId, Point),
    Divider(DividerDrag),
}

impl Running {
    pub(super) fn drag_pressed(&mut self, hit: Hit) {
        let Some(origin) = self.pointer.position else {
            return;
        };
        self.drag = match hit {
            Hit::Tab(_, tab) => Some(Drag::Pending(tab, origin)),
            Hit::Divider(index) => self.divider_drag(index, origin).map(Drag::Divider),
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

    pub(super) fn drag_moved(&mut self, point: Point) -> bool {
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
        };
        self.drag = Some(next);
        self.changed();
        true
    }

    pub(super) fn drag_released(&mut self) -> bool {
        match self.drag.take() {
            Some(Drag::Tab(tab, pointer)) => {
                if let Some(spot) = self.drop_spot(tab, pointer) {
                    self.workspace.move_tab(tab, spot.target);
                    self.motion.animate_next();
                }
                true
            }
            Some(Drag::Divider(_)) => true,
            _ => false,
        }
    }

    pub(super) fn drag_key(&mut self, key: &Key) -> bool {
        *key == Key::Named(NamedKey::Escape) && self.cancel_drag()
    }

    pub(super) fn cancel_drag(&mut self) -> bool {
        let drag = match self.drag.take() {
            None | Some(Drag::Pending(..)) => return false,
            Some(drag) => drag,
        };
        if let Drag::Divider(drag) = drag
            && let Some(instance) = self.workspace.active_instance_mut()
        {
            instance.root = drag.root;
        }
        self.pointer.pressed = None;
        self.view.pressed = None;
        self.changed();
        true
    }

    pub(super) fn drags_tab(&self) -> bool {
        matches!(self.drag, Some(Drag::Tab(..)))
    }

    pub(super) fn drag_cursor(&self) -> Option<CursorIcon> {
        match &self.drag {
            Some(Drag::Divider(drag)) => Some(match drag.divider.axis {
                Axis::Row => CursorIcon::ColResize,
                Axis::Column => CursorIcon::RowResize,
            }),
            Some(Drag::Tab(..)) => Some(CursorIcon::Default),
            _ => None,
        }
    }

    pub(super) fn drag_view(&self) -> Option<DragView> {
        let Some(Drag::Tab(tab, pointer)) = self.drag else {
            return None;
        };
        Some(DragView {
            label: self.tab_title(tab),
            pointer,
            mark: self.drop_spot(tab, pointer).map(|spot| spot.mark),
        })
    }

    fn drop_spot(&self, tab: SessionId, pointer: Point) -> Option<DropSpot> {
        let instance = self.workspace.active_instance()?.id;
        let spot = self.frame.drop_at(pointer, instance)?;
        let source = self.workspace.locate(tab)?;
        let same_pane = matches!(
            spot.target,
            DropTarget::Tabs { pane, index: None, .. } if pane == source.pane
        );
        if same_pane {
            return None;
        }
        self.workspace
            .clone()
            .move_tab(tab, spot.target)
            .then_some(spot)
    }
}

fn distance(from: Point, to: Point) -> f32 {
    (to.x - from.x).hypot(to.y - from.y)
}
