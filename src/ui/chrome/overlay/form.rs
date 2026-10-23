use super::ON_ACCENT;
use super::editor::draw_field;
use crate::ui::chrome::Painter;
use crate::ui::frame::{EditorFrame, FormFrame, FormRow};
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

pub(super) fn draw_connect_dialog(painter: &mut Painter, editor: &EditorFrame) {
    let Some(form) = &editor.form else {
        return;
    };
    let theme = painter.theme;
    for section in &editor.sections {
        painter.setting_line(section, theme.text_faint);
    }
    let radius = painter.metrics.px(UiMetrics::RADIUS);
    for (index, line) in editor.hosts.iter().enumerate() {
        let glow = painter.press_fill(line.rect, Hit::EditHost(index), radius);
        painter.setting_line(line, theme.muted_text(glow));
    }
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
