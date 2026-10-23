use std::sync::Arc;

use crate::geometry::Rect;
use crate::input::TextField;
use crate::locale::Strings;
use crate::preferences::{ConnectForm, FormField};
use crate::text::{TextLayout, TextSystem};
use crate::ui::frame::editor::{FieldFrame, place_field};
use crate::ui::frame::sidebar::LINE_INSET;
use crate::ui::metrics::UiMetrics;

pub const SECRET_MASK: char = '\u{2022}';

const LABEL_GAP: f32 = 10.0;

#[derive(Clone, Debug)]
pub struct FormRow {
    pub field: FormField,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
    pub rect: Rect,
    pub input: FieldFrame,
    pub focused: bool,
}

#[derive(Clone, Copy)]
struct FormGrid {
    area: Rect,
    label_x: f32,
    field_x: f32,
}

fn label_of(strings: &'static Strings, field: FormField) -> &'static str {
    match field {
        FormField::Address => strings.ssh_address,
        FormField::User => strings.ssh_user,
        FormField::Port => strings.ssh_port,
        FormField::Secret => strings.ssh_password,
    }
}

fn shown(form: &ConnectForm, field: FormField) -> TextField {
    match field {
        FormField::Secret => form.field(field).masked(SECRET_MASK),
        _ => form.field(field).clone(),
    }
}

pub(super) fn form_rows(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    area: Rect,
    form: &ConnectForm,
    strings: &'static Strings,
) -> Vec<FormRow> {
    let labels = FormField::ALL.map(|field| text.layout(label_of(strings, field), metrics.label()));
    let label_width = labels
        .iter()
        .map(|label| label.width)
        .fold(0.0, f32::max)
        .ceil();
    let label_x = area.x + metrics.px(LINE_INSET);
    let grid = FormGrid {
        area,
        label_x,
        field_x: label_x + label_width + metrics.px(LABEL_GAP),
    };
    placed_rows(text, metrics, form, grid, labels)
}

fn placed_rows(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    form: &ConnectForm,
    FormGrid {
        area,
        label_x,
        field_x,
    }: FormGrid,
    labels: [Arc<TextLayout>; FormField::ALL.len()],
) -> Vec<FormRow> {
    let row_height = metrics.px(UiMetrics::SIDEBAR_ROW);
    let place = |(index, (field, label)): (usize, (FormField, Arc<TextLayout>))| {
        let y = area.y + index as f32 * row_height;
        let rect = Rect::new(area.x, y, area.width, row_height);
        let slot = Rect::from_edges(field_x, y, rect.right(), rect.bottom());
        FormRow {
            field,
            label,
            label_x,
            rect,
            input: place_field(text, metrics, slot, area, &shown(form, field)),
            focused: form.focus() == field,
        }
    };
    FormField::ALL
        .into_iter()
        .zip(labels)
        .enumerate()
        .map(place)
        .collect()
}
