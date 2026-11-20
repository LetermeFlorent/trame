use trame::geometry::Point;
use trame::input::{
    MouseAction, MouseButton, MouseEvent, MouseModifiers, WheelAction, arrow_keys, encode_mouse,
    wheel_action,
};
use trame::session::SessionId;
use trame::term::{Modes, Screen};
use trame::ui::Motion;
use winit::keyboard::ModifiersState;

use crate::gnome::state::WindowState;
use crate::gnome::state::settings::ListKind;

const WHEEL_LINES: f32 = 3.0;

pub fn scroll_lines(state: &mut WindowState, dy: f32, pixel: bool) {
    let list = state.list_under_pointer(ListKind::Hosts);
    if state.picture_open || (list.is_none() && (state.frame.blocked || state.settings.is_some())) {
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
        .find(|p| p.body.contains(point))?
        .active?;
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
            scroll_history(
                &mut state.motion,
                id,
                terminal.screen_mut(),
                n,
                state.desk.preferences.smooth_scroll,
            );
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
            (
                ((point.x - p.terminal.x).max(0.0) / state.cell_size.0.max(1.0)) as usize,
                ((point.y - p.terminal.y).max(0.0) / state.cell_size.1.max(1.0)) as usize,
            )
        });
    let event = MouseEvent {
        action: MouseAction::Press,
        button: Some(if lines > 0 {
            MouseButton::WheelUp
        } else {
            MouseButton::WheelDown
        }),
        col: cell.0,
        row: cell.1,
        modifiers: mouse_mods(state.modifiers),
    };
    encode_mouse(&event, modes)
        .unwrap_or_default()
        .repeat(lines.unsigned_abs().min(10))
}

fn mouse_mods(m: ModifiersState) -> MouseModifiers {
    MouseModifiers {
        shift: m.shift_key(),
        alt: m.alt_key(),
        ctrl: m.control_key(),
    }
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
