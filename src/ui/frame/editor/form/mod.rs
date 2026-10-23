mod buttons;
mod rows;

use crate::geometry::{Point, Rect};
use crate::locale::Strings;
use crate::preferences::{ConnectForm, FormField};
use crate::text::TextSystem;
use crate::ui::metrics::UiMetrics;
use buttons::place_buttons;
pub use buttons::{FormAction, FormButton};
use rows::form_rows;
pub use rows::{FormRow, SECRET_MASK};

const FORM_PAD: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct FormFrame {
    pub rect: Rect,
    pub rows: Vec<FormRow>,
    pub buttons: Vec<FormButton>,
}

impl FormFrame {
    pub fn field_at(&self, point: Point) -> Option<FormField> {
        self.rows
            .iter()
            .find(|row| row.input.rect.contains(point))
            .map(|row| row.field)
    }

    pub fn button_at(&self, point: Point) -> Option<FormAction> {
        self.buttons
            .iter()
            .find(|button| button.rect.contains(point))
            .map(|button| button.action)
    }
}

pub(in crate::ui::frame) struct FormSpec<'a> {
    pub(in crate::ui::frame) area: Rect,
    pub(in crate::ui::frame) form: &'a ConnectForm,
    pub(in crate::ui::frame) actions: &'a [FormAction],
    pub(in crate::ui::frame) strings: &'static Strings,
}

pub(in crate::ui::frame) fn form_title(strings: &'static Strings, changing: bool) -> &'static str {
    match changing {
        true => strings.change_connection,
        false => strings.new_connection,
    }
}

pub(in crate::ui::frame) fn form_height(metrics: &UiMetrics) -> f32 {
    FormField::ALL.len() as f32 * metrics.px(UiMetrics::SIDEBAR_ROW)
        + metrics.px(FORM_PAD)
        + metrics.px(UiMetrics::BUTTON)
}

pub(in crate::ui::frame) fn button_form(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    area: Rect,
    actions: &[FormAction],
    strings: &'static Strings,
) -> FormFrame {
    FormFrame {
        rect: area,
        rows: Vec::new(),
        buttons: place_buttons(text, metrics, area, actions, strings),
    }
}

pub(in crate::ui::frame) fn place_form(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    spec: &FormSpec,
) -> FormFrame {
    let FormSpec {
        area,
        form,
        actions,
        strings,
    } = *spec;
    FormFrame {
        rect: area,
        rows: form_rows(text, metrics, area, form, strings),
        buttons: place_buttons(text, metrics, area, actions, strings),
    }
}
