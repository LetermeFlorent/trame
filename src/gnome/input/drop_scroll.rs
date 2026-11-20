use std::path::Path;

use trame::geometry::Point;
use trame::ui::Hit;
use trame::workspace::PaneId;

use crate::gnome::state::WindowState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropOutcome {
    Ignored,
    Focused,
    Pane(PaneId),
}

pub fn drop_target(hit: Option<Hit>, modal: bool) -> DropOutcome {
    match (modal, hit) {
        (true, _) => DropOutcome::Ignored,
        (false, Some(Hit::Terminal(pane))) => DropOutcome::Pane(pane),
        (false, _) => DropOutcome::Focused,
    }
}

pub fn dropped_text<'a>(paths: impl IntoIterator<Item = &'a Path>) -> String {
    paths
        .into_iter()
        .filter_map(|p| p.to_str())
        .filter(|s| !s.is_empty() && !s.chars().any(char::is_control))
        .map(|s| format!("{s} "))
        .collect()
}

pub fn apply_file_drop(state: &mut WindowState, point: Option<Point>, paths: &[&Path]) -> bool {
    let text = dropped_text(paths.iter().copied());
    if text.is_empty() {
        return false;
    }
    let hit = point.and_then(|p| state.frame.hit(p, false));
    match drop_target(hit, state.modal_open()) {
        DropOutcome::Ignored => return false,
        DropOutcome::Pane(pane) => state.desk.focus_pane(pane),
        DropOutcome::Focused => {}
    }
    let effects = state.desk.write_to_active(|_| text.into_bytes());
    state.apply(effects);
    state.changed();
    true
}
