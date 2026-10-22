mod zones;

use crate::geometry::Point;
use crate::session::SessionId;
use crate::theme::{Emblem, Swatch};
use crate::ui::frame::{FrameLayout, SidebarItem};
use crate::ui::metrics::UiMetrics;
use crate::workspace::{Axis, InstanceId, PaneId};
use zones::{hit_editor, hit_pane, hit_sidebar, hit_title_bar};

const CORNER_FACTOR: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeEdge {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Resize(ResizeEdge),
    TitleBar,
    SidebarToggle,
    RailInstance(InstanceId),
    SidebarEdge,
    Minimize,
    Maximize,
    Close,
    Sidebar,
    NewInstance,
    Row(SidebarItem),
    RowClose(SidebarItem),
    TabBar(PaneId),
    Tab(PaneId, SessionId),
    TabClose(PaneId, SessionId),
    NewTab(PaneId),
    Split(PaneId),
    Divider(usize),
    Terminal(PaneId),
    EditField,
    EditSwatch(Option<Swatch>),
    EditEmblem(Emblem),
}

impl FrameLayout {
    pub fn hit(&self, point: Point, resizable: bool) -> Option<Hit> {
        if !self.bounds.contains(point) {
            return None;
        }
        if let Some(edge) = resizable.then(|| self.resize_edge(point)).flatten() {
            return Some(Hit::Resize(edge));
        }
        if let Some(hit) = self
            .editor
            .as_ref()
            .and_then(|editor| hit_editor(editor, point))
        {
            return Some(hit);
        }
        if self.title_bar.rect.contains(point) {
            return Some(hit_title_bar(&self.title_bar, point));
        }
        if self.sidebar_edge().contains(point) {
            return Some(Hit::SidebarEdge);
        }
        if let Some(sidebar) = self
            .sidebar
            .as_ref()
            .filter(|side| side.rect.contains(point))
        {
            return Some(hit_sidebar(sidebar, point));
        }
        if let Some(rail) = self.rail.as_ref().filter(|rail| rail.rect.contains(point)) {
            return Some(
                rail.slot_at(point)
                    .map_or(Hit::Sidebar, |slot| Hit::RailInstance(slot.id)),
            );
        }
        if let Some(index) = self.divider_at(point) {
            return Some(Hit::Divider(index));
        }
        self.panes
            .iter()
            .find(|pane| pane.rect.contains(point))
            .map(|pane| hit_pane(pane, point))
    }

    pub fn divider_at(&self, point: Point) -> Option<usize> {
        let grab = self.metrics.px(UiMetrics::DIVIDER_GRAB);
        self.dividers.iter().position(|divider| {
            let zone = match divider.axis {
                Axis::Row => divider.rect.inset_by(-grab, 0.0),
                Axis::Column => divider.rect.inset_by(0.0, -grab),
            };
            zone.contains(point)
        })
    }

    fn resize_edge(&self, point: Point) -> Option<ResizeEdge> {
        let border = self.metrics.px(UiMetrics::RESIZE_BORDER);
        let corner = border * CORNER_FACTOR;
        let bounds = self.bounds;
        let near = |distance: f32, reach: f32| distance < reach;
        let west = near(point.x - bounds.x, border);
        let east = near(bounds.right() - point.x, border);
        let north = near(point.y - bounds.y, border);
        let south = near(bounds.bottom() - point.y, border);
        let west_corner = near(point.x - bounds.x, corner);
        let east_corner = near(bounds.right() - point.x, corner);
        let north_corner = near(point.y - bounds.y, corner);
        let south_corner = near(bounds.bottom() - point.y, corner);
        match () {
            _ if (north && west_corner) || (west && north_corner) => Some(ResizeEdge::NorthWest),
            _ if (north && east_corner) || (east && north_corner) => Some(ResizeEdge::NorthEast),
            _ if (south && west_corner) || (west && south_corner) => Some(ResizeEdge::SouthWest),
            _ if (south && east_corner) || (east && south_corner) => Some(ResizeEdge::SouthEast),
            _ if north => Some(ResizeEdge::North),
            _ if south => Some(ResizeEdge::South),
            _ if west => Some(ResizeEdge::West),
            _ if east => Some(ResizeEdge::East),
            _ => None,
        }
    }
}
