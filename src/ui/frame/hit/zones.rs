use super::Hit;
use crate::geometry::{Point, Rect};
use crate::ui::frame::{EditorFrame, PaneFrame, SidebarFrame, TitleBarFrame};

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

pub(super) fn hit_editor(editor: &EditorFrame, point: Point) -> Option<Hit> {
    if let Some(slot) = editor.slot_at(point) {
        return Some(Hit::EditSwatch(slot.color));
    }
    editor.contains(point).then_some(Hit::EditField)
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
