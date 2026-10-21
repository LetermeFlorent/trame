use crate::geometry::{Point, Rect};
use crate::session::SessionId;
use crate::ui::frame::{FrameLayout, PaneFrame, SidebarFrame, SidebarItem, TitleBarFrame};
use crate::ui::metrics::UiMetrics;
use crate::workspace::{Axis, PaneId};

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
}

impl FrameLayout {
    pub fn hit(&self, point: Point, resizable: bool) -> Option<Hit> {
        if !self.bounds.contains(point) {
            return None;
        }
        if let Some(edge) = resizable.then(|| self.resize_edge(point)).flatten() {
            return Some(Hit::Resize(edge));
        }
        if self.title_bar.rect.contains(point) {
            return Some(hit_title_bar(&self.title_bar, point));
        }
        if let Some(sidebar) = self.sidebar.as_ref().filter(|side| side.rect.contains(point)) {
            return Some(hit_sidebar(sidebar, point));
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

fn hit_title_bar(bar: &TitleBarFrame, point: Point) -> Hit {
    if bar.toggle.contains(point) {
        return Hit::SidebarToggle;
    }
    let Some(controls) = bar.controls else {
        return Hit::TitleBar;
    };
    let buttons = [
        (controls.minimize, Hit::Minimize),
        (controls.maximize, Hit::Maximize),
        (controls.close, Hit::Close),
    ];
    first_hit(&buttons, point).unwrap_or(Hit::TitleBar)
}

fn hit_sidebar(sidebar: &SidebarFrame, point: Point) -> Hit {
    if sidebar.new_instance.contains(point) {
        return Hit::NewInstance;
    }
    let Some(row) = sidebar.rows.iter().find(|row| row.rect.contains(point)) else {
        return Hit::Sidebar;
    };
    match row.close.contains(point) {
        true => Hit::RowClose(row.item),
        false => Hit::Row(row.item),
    }
}

fn hit_pane(pane: &PaneFrame, point: Point) -> Hit {
    let strip = &pane.strip;
    if !strip.rect.contains(point) {
        return Hit::Terminal(pane.id);
    }
    let buttons = [
        (strip.new_tab, Hit::NewTab(pane.id)),
        (strip.split, Hit::Split(pane.id)),
    ];
    if let Some(hit) = first_hit(&buttons, point) {
        return hit;
    }
    let tab = strip
        .tabs
        .iter()
        .filter(|_| strip.tabs_area.contains(point))
        .find(|tab| tab.rect.contains(point));
    match tab {
        Some(tab) if tab.close.contains(point) => Hit::TabClose(pane.id, tab.id),
        Some(tab) => Hit::Tab(pane.id, tab.id),
        None => Hit::TabBar(pane.id),
    }
}

fn first_hit(candidates: &[(Rect, Hit)], point: Point) -> Option<Hit> {
    candidates
        .iter()
        .find(|(rect, _)| rect.contains(point))
        .map(|(_, hit)| *hit)
}
