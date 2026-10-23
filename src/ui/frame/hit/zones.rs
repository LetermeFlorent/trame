use super::{Hit, ResizeEdge};
use crate::geometry::{Point, Rect};
use crate::ui::frame::{MenuFrame, PaneFrame, SettingsFrame, SidebarFrame, TitleBarFrame};

const CORNER_FACTOR: f32 = 2.0;

pub(super) fn hit_title_bar(bar: &TitleBarFrame, point: Point) -> Hit {
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

pub(super) fn hit_sidebar(sidebar: &SidebarFrame, point: Point) -> Hit {
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

pub(super) fn hit_menu(menu: &MenuFrame, point: Point) -> Option<Hit> {
    menu.rect
        .contains(point)
        .then(|| menu.entry_at(point).map_or(Hit::Menu, Hit::MenuEntry))
}

pub(super) fn hit_settings(panel: &SettingsFrame, point: Point) -> Option<Hit> {
    match panel.popup.rect.contains(point) {
        true => Some(
            panel
                .control_at(point)
                .map_or(Hit::SettingsPanel, Hit::Setting),
        ),
        false => panel
            .popup
            .backdrop
            .contains(point)
            .then_some(Hit::Backdrop),
    }
}

pub(super) fn hit_pane(pane: &PaneFrame, point: Point) -> Hit {
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

pub(super) fn resize_edge(bounds: Rect, border: f32, point: Point) -> Option<ResizeEdge> {
    let corner = border * CORNER_FACTOR;
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

pub(super) fn through_backdrop(hit: Option<Hit>) -> Hit {
    match hit {
        Some(
            hit @ (Hit::Resize(_) | Hit::TitleBar | Hit::Minimize | Hit::Maximize | Hit::Close),
        ) => hit,
        _ => Hit::Backdrop,
    }
}
