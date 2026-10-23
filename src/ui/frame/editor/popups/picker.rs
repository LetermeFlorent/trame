use super::EditorFrame;
use crate::geometry::{Point, Rect};
use crate::input::TextField;
use crate::locale::Strings;
use crate::text::TextSystem;
use crate::theme::Hsv;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::editor::{FieldFrame, FormAction, button_form, place_field};
use crate::ui::frame::sidebar::SECTION_GAP;
use crate::ui::metrics::UiMetrics;

const SQUARE_HEIGHT: f32 = 150.0;
const HUE_WIDTH: f32 = 16.0;
const HUE_GAP: f32 = 10.0;
const FIELD_WIDTH: f32 = 96.0;
const PREVIEW_GAP: f32 = 8.0;
const PICKER_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Apply];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerPart {
    Square,
    Hue,
    Field,
}

#[derive(Clone, Copy, Debug)]
pub struct PickerView<'a> {
    pub hsv: Hsv,
    pub field: &'a TextField,
    pub editing: bool,
}

#[derive(Clone, Debug)]
pub struct PickerFrame {
    pub square: Rect,
    pub hue: Rect,
    pub hsv: Hsv,
    pub preview: Rect,
    pub field: FieldFrame,
}

impl PickerFrame {
    pub fn part_at(&self, point: Point) -> Option<PickerPart> {
        match () {
            _ if self.square.contains(point) => Some(PickerPart::Square),
            _ if self.hue.contains(point) => Some(PickerPart::Hue),
            _ if self.field.rect.contains(point) => Some(PickerPart::Field),
            _ => None,
        }
    }

    pub fn picked(&self, part: PickerPart, point: Point, hsv: Hsv) -> Hsv {
        let (square, hue) = (self.square, self.hue);
        match part {
            PickerPart::Square => Hsv {
                saturation: share(point.x, square.x, square.width),
                value: 1.0 - share(point.y, square.y, square.height),
                ..hsv
            },
            PickerPart::Hue => Hsv {
                hue: share(point.y, hue.y, hue.height),
                ..hsv
            },
            PickerPart::Field => hsv,
        }
    }
}

fn share(at: f32, from: f32, length: f32) -> f32 {
    ((at - from) / length.max(1.0)).clamp(0.0, 1.0)
}

impl FrameLayout {
    pub(in crate::ui::frame::editor) fn picker_popup(
        &self,
        text: &mut TextSystem,
        strings: &'static Strings,
        view: PickerView,
        editor: EditorFrame,
    ) -> EditorFrame {
        let metrics = self.metrics;
        let row = metrics.px(UiMetrics::SIDEBAR_ROW);
        let height = metrics.px(SQUARE_HEIGHT);
        let content = height + metrics.px(SECTION_GAP) + row;
        let title = match view.editing {
            true => strings.edit_color,
            false => strings.new_color,
        };
        let (popup, column) = self.open_popup(text, title, content);
        let inner = column.inner;
        let hue_width = metrics.px(HUE_WIDTH);
        let square_width = inner.width - hue_width - metrics.px(HUE_GAP);
        let square = Rect::new(inner.x, column.y, square_width, height);
        let hue = Rect::new(inner.right() - hue_width, column.y, hue_width, height);
        let top = square.bottom() + metrics.px(SECTION_GAP);
        let side = metrics.px(UiMetrics::BUTTON);
        let preview = Rect::new(inner.x, (top + (row - side) / 2.0).round(), side, side);
        let field_x = preview.right() + metrics.px(PREVIEW_GAP);
        let slot = Rect::new(field_x, top, metrics.px(FIELD_WIDTH), row);
        let field = place_field(column.text, &metrics, slot, inner, view.field);
        let area = Rect::new(inner.x, preview.y, inner.width, side);
        let form = button_form(column.text, &metrics, area, &PICKER_ACTIONS, strings);
        let picker = PickerFrame {
            square,
            hue,
            hsv: view.hsv,
            preview,
            field,
        };
        EditorFrame {
            popup: Some(popup),
            picker: Some(picker),
            form: Some(form),
            ..editor
        }
    }
}
