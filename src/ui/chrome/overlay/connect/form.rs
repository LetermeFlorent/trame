use crate::ui::chrome::Painter;
use crate::ui::chrome::overlay::editor::draw_field;
use crate::ui::chrome::overlay::popup::{LabelButton, draw_label_button};
use crate::ui::frame::{FormFrame, FormRow};
use crate::ui::hit::Hit;

pub(in crate::ui::chrome::overlay) fn draw_form(painter: &mut Painter, form: &FormFrame) {
    for row in &form.rows {
        draw_row(painter, row);
    }
    for button in &form.buttons {
        let label = LabelButton {
            rect: button.rect,
            label: &button.label,
            hit: Hit::FormButton(button.action),
            filled: button.action.is_primary(),
            faint: false,
        };
        draw_label_button(painter, &label);
    }
}

fn draw_row(painter: &mut Painter, row: &FormRow) {
    let color = match row.focused {
        true => painter.theme.text,
        false => painter.theme.text_muted,
    };
    painter.label(&row.label, row.label_x, row.rect, color);
    draw_field(painter, &row.input, row.focused);
}
