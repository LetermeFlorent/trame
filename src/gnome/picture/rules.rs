use trame::geometry::Rect;
use trame::term::Modes;
use trame::ui::terminal::links::LinkSpan;

pub const MAX_OVERLAY_SIDE: u32 = 2048;
pub const MAX_PREVIEW_SIDE: u32 = 2047;

pub fn reports_mouse(modes: Modes, shift_held: bool) -> bool {
    modes.intersects(Modes::MOUSE_ANY) && !shift_held
}

pub fn inline_click_opens(reports: bool, ctrl_held: bool) -> bool {
    !reports || ctrl_held
}

pub fn link_active(overlay_open: bool, reports: bool, ctrl_held: bool) -> bool {
    !overlay_open && !reports && ctrl_held
}

pub fn grid_cell(
    point: (f32, f32),
    cell_size: (f32, f32),
    grid_size: (usize, usize),
) -> Option<(usize, usize)> {
    if cell_size.0 <= 0.0 || cell_size.1 <= 0.0 || point.0 < 0.0 || point.1 < 0.0 {
        return None;
    }
    let col = (point.0 / cell_size.0).floor() as usize;
    let row = (point.1 / cell_size.1).floor() as usize;
    (row < grid_size.0 && col < grid_size.1).then_some((row, col))
}

pub fn span_anchor(span: &LinkSpan, cell_size: (f32, f32)) -> Rect {
    Rect::new(
        span.start as f32 * cell_size.0,
        span.row as f32 * cell_size.1,
        (span.end - span.start) as f32 * cell_size.0,
        cell_size.1,
    )
}
