use super::ON_ACCENT;
use super::editor::{STRIP_BLUR, draw_field};
use crate::ui::chrome::Painter;
use crate::ui::frame::{FormFrame, FormRow};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

pub(super) fn draw_form(painter: &mut Painter, form: &FormFrame) {
    painter.panel(form.rect, STRIP_BLUR);
    for row in &form.rows {
        draw_row(painter, row);
    }
    draw_connect(painter, form);
}

fn draw_row(painter: &mut Painter, row: &FormRow) {
    let color = match row.focused {
        true => painter.theme.text,
        false => painter.theme.text_muted,
    };
    painter.label(&row.label, row.label_x, row.rect, color);
    draw_field(painter, &row.input, row.focused);
}

fn draw_connect(painter: &mut Painter, form: &FormFrame) {
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    painter
        .scene
        .fill(form.connect, painter.theme.accent, radius);
    painter.press_fill(form.connect, Hit::FormConnect, radius);
    let x = form.connect.centered_x(form.connect_label.width);
    painter.label(&form.connect_label, x, form.connect, ON_ACCENT);
}
