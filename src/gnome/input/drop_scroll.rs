use std::path::Path;

use trame::geometry::Point;
use trame::input::{MouseAction, WheelAction, arrow_keys, encode_mouse, wheel_action};
use trame::session::SessionId;
use trame::term::{Modes, Screen};
use trame::ui::{Hit, Motion};
use trame::workspace::PaneId;

use crate::gnome::state::WindowState;
use crate::gnome::state::settings::ListKind;

const WHEEL_LINES: f32 = 3.0;

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

pub fn scroll_lines(state: &mut WindowState, dy: f32, pixel: bool) {
    let list = state.list_under_pointer(ListKind::Hosts);
    if state.picture_open || (list.is_none() && (state.frame.blocked || state.settings.is_some()))
    {
        return;
    }
    let step = if pixel {
        -dy / state.cell_size.1.max(1.0)
    } else {
        -dy * WHEEL_LINES
    };
    state.wheel += step;
    let whole = state.wheel.trunc();
    state.wheel -= whole;
    if whole == 0.0 {
        return;
    }
    let delta = whole as isize;
    if let Some(bar) = list {
        return state.scroll_list_to(ListKind::Hosts, bar.scrolled(-delta));
    }
    if state.editing.is_none()
        && let Some(bar) = state.list_under_pointer(ListKind::Sidebar)
    {
        return state.scroll_list_to(ListKind::Sidebar, bar.scrolled(-delta));
    }
    if let Some((id, p)) = session_at_pointer(state) {
        scroll_session(state, id, delta, p);
    }
}

fn session_at_pointer(state: &WindowState) -> Option<(SessionId, Point)> {
    let point = state.pointer?;
    let id = state
        .frame
        .panes
        .iter()
        .find(|p| p.body.contains(point))
        .and_then(|p| p.active)?;
    Some((id, point))
}

fn scroll_session(state: &mut WindowState, id: SessionId, lines: isize, point: Point) {
    let Some(session) = state.desk.sessions.get(id) else {
        return;
    };
    let mut terminal = session.terminal();
    let modes = terminal.screen().modes();
    match wheel_action(lines, modes) {
        WheelAction::History(n) => {
            let smooth = state.desk.preferences.smooth_scroll;
            scroll_history(&mut state.motion, id, terminal.screen_mut(), n, smooth);
            drop(terminal);
            state.dirty = true;
        }
        WheelAction::Keys(count, up) => {
            drop(terminal);
            session.write(arrow_keys(count, up, modes));
        }
        WheelAction::Report => {
            let bytes = wheel_packets(state, id, lines, point, modes);
            drop(terminal);
            session.write(bytes);
        }
        WheelAction::Ignore => {}
    }
}

fn wheel_packets(
    state: &WindowState,
    id: SessionId,
    lines: isize,
    point: Point,
    modes: Modes,
) -> Vec<u8> {
    let cell = state
        .frame
        .panes
        .iter()
        .find(|p| p.active == Some(id))
        .map_or((0, 0), |p| {
            let col = ((point.x - p.terminal.x).max(0.0) / state.cell_size.0.max(1.0)) as usize;
            let row = ((point.y - p.terminal.y).max(0.0) / state.cell_size.1.max(1.0)) as usize;
            (col, row)
        });
    let act = if lines > 0 {
        MouseAction::WheelUp
    } else {
        MouseAction::WheelDown
    };
    let mut out = Vec::new();
    for _ in 0..lines.unsigned_abs() {
        if let Some(pkt) = encode_mouse(act, None, cell.0, cell.1, state.modifiers, modes) {
            out.extend_from_slice(&pkt);
        }
    }
    out
}

pub fn scroll_history(
    motion: &mut Motion,
    id: SessionId,
    screen: &mut Screen,
    lines: isize,
    smooth: bool,
) {
    let before = screen.grid().display_offset();
    screen.scroll_display(lines);
    let after = screen.grid().display_offset();
    match smooth {
        true => motion.scroll_by(id, before as f32 - after as f32, screen.rows() as f32),
        false => motion.stop_scroll(id),
    }
}
