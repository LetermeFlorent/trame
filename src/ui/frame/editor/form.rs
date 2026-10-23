use std::sync::Arc;

use super::{FieldFrame, place_field};
use crate::geometry::{Point, Rect};
use crate::input::TextField;
use crate::locale::Strings;
use crate::preferences::{ConnectForm, FormField};
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::UiMetrics;

pub const SECRET_MASK: char = '\u{2022}';

const FORM_PAD: f32 = 8.0;
const LABEL_GAP: f32 = 10.0;
const BUTTON_PAD_X: f32 = 12.0;

#[derive(Clone, Debug)]
pub struct FormRow {
    pub field: FormField,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
    pub rect: Rect,
    pub input: FieldFrame,
    pub focused: bool,
}

#[derive(Clone, Debug)]
pub struct FormFrame {
    pub rect: Rect,
    pub rows: Vec<FormRow>,
    pub connect: Rect,
    pub connect_label: Arc<TextLayout>,
}

impl FormFrame {
    pub fn field_at(&self, point: Point) -> Option<FormField> {
        self.rows
            .iter()
            .find(|row| row.input.rect.contains(point))
            .map(|row| row.field)
    }
}

#[derive(Clone, Copy)]
struct FormGrid {
    area: Rect,
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

pub(super) fn form_height(metrics: &UiMetrics) -> f32 {
    FormField::ALL.len() as f32 * metrics.px(UiMetrics::SIDEBAR_ROW)
        + metrics.px(FORM_PAD)
        + metrics.px(UiMetrics::BUTTON)
}

pub(super) fn place_form(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    area: Rect,
    form: &ConnectForm,
    strings: &'static Strings,
) -> FormFrame {
    let labels = FormField::ALL.map(|field| text.layout(label_of(strings, field), metrics.label()));
    let label_width = labels
        .iter()
        .map(|label| label.width)
        .fold(0.0, f32::max)
        .ceil();
    let grid = FormGrid {
        area,
        field_x: area.x + label_width + metrics.px(LABEL_GAP),
    };
    let rows = form_rows(text, metrics, form, grid, labels);
    let connect_label = text.layout(strings.connect, metrics.label());
    let connect_width = connect_label.width.ceil() + 2.0 * metrics.px(BUTTON_PAD_X);
    let button_height = metrics.px(UiMetrics::BUTTON);
    let connect = Rect::new(
        area.right() - connect_width,
        area.bottom() - button_height,
        connect_width,
        button_height,
    );
    FormFrame {
        rect: area,
        rows,
        connect,
        connect_label,
    }
}

fn form_rows(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    form: &ConnectForm,
    FormGrid { area, field_x }: FormGrid,
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
            label_x: rect.x,
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
