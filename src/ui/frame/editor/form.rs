use std::sync::Arc;

use super::row::RowSize;
use super::{FieldFrame, place_field, place_strip};
use crate::geometry::{Point, Rect};
use crate::input::TextField;
use crate::locale::Strings;
use crate::preferences::{ConnectForm, FormField};
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::UiMetrics;

pub const SECRET_MASK: char = '\u{2022}';

const FORM_PAD: f32 = 8.0;
const LABEL_GAP: f32 = 10.0;
const FIELD_WIDTH: f32 = 190.0;
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
    inner: Rect,
    field_x: f32,
}

pub(super) struct FormSpot<'a> {
    pub(super) anchor: Rect,
    pub(super) bounds: Rect,
    pub(super) form: &'a ConnectForm,
    pub(super) strings: &'static Strings,
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

pub(super) fn place_form(text: &mut TextSystem, metrics: &UiMetrics, spot: &FormSpot) -> FormFrame {
    let pad = metrics.px(FORM_PAD);
    let labels =
        FormField::ALL.map(|field| text.layout(label_of(spot.strings, field), metrics.label()));
    let label_width = labels
        .iter()
        .map(|label| label.width)
        .fold(0.0, f32::max)
        .ceil();
    let columns = label_width + metrics.px(LABEL_GAP) + metrics.px(FIELD_WIDTH);
    let rows_height = FormField::ALL.len() as f32 * metrics.px(UiMetrics::SIDEBAR_ROW);
    let size = RowSize {
        width: (columns + 2.0 * pad).min(spot.bounds.width),
        height: rows_height + metrics.px(UiMetrics::BUTTON) + 3.0 * pad,
    };
    let rect = place_strip(spot.anchor, size, spot.bounds);
    let inner = rect.inset(pad);
    let grid = FormGrid {
        inner,
        field_x: inner.x + label_width + metrics.px(LABEL_GAP),
    };
    let rows = form_rows(text, metrics, spot, grid, labels);
    let connect_label = text.layout(spot.strings.connect, metrics.label());
    let connect_width = connect_label.width.ceil() + 2.0 * metrics.px(BUTTON_PAD_X);
    let button_height = metrics.px(UiMetrics::BUTTON);
    let connect = Rect::new(
        inner.right() - connect_width,
        inner.bottom() - button_height,
        connect_width,
        button_height,
    );
    FormFrame {
        rect,
        rows,
        connect,
        connect_label,
    }
}

fn form_rows(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    spot: &FormSpot,
    FormGrid { inner, field_x }: FormGrid,
    labels: [Arc<TextLayout>; FormField::ALL.len()],
) -> Vec<FormRow> {
    let row_height = metrics.px(UiMetrics::SIDEBAR_ROW);
    let place = |(index, (field, label)): (usize, (FormField, Arc<TextLayout>))| {
        let y = inner.y + index as f32 * row_height;
        let rect = Rect::new(inner.x, y, inner.width, row_height);
        let slot = Rect::from_edges(field_x, y, rect.right(), rect.bottom());
        FormRow {
            field,
            label,
            label_x: rect.x,
            rect,
            input: place_field(text, metrics, slot, spot.bounds, &shown(spot.form, field)),
            focused: spot.form.focus() == field,
        }
    };
    FormField::ALL
        .into_iter()
        .zip(labels)
        .enumerate()
        .map(place)
        .collect()
}
