use crate::geometry::{Point, Rect};
use crate::locale::Strings;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::editor::{FormAction, FormFrame, place_buttons};
use crate::ui::metrics::UiMetrics;

use super::column::{LINE_HEIGHT, SECTION_GAP};
use super::{PopupFrame, SettingLine};

const CONFIRM_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Close];

#[derive(Clone, Debug)]
pub struct ConfirmFrame {
    pub popup: PopupFrame,
    pub message: SettingLine,
    pub form: FormFrame,
}

pub struct ConfirmRequest<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub strings: &'static Strings,
}

impl ConfirmFrame {
    pub fn button_at(&self, point: Point) -> Option<FormAction> {
        self.form.button_at(point)
    }
}

impl FrameLayout {
    pub fn place_confirm(&mut self, text: &mut TextSystem, request: &ConfirmRequest) {
        let metrics = self.metrics;
        let actions = metrics.px(SECTION_GAP) + metrics.px(UiMetrics::BUTTON);
        let content = metrics.px(LINE_HEIGHT) + actions;
        let (popup, mut column) = self.open_popup(text, request.title, content);
        let message = column.line(request.message, LINE_HEIGHT, metrics.label());
        let area = Rect::new(column.inner.x, column.y, column.inner.width, actions);
        let buttons = place_buttons(
            column.text,
            &metrics,
            area,
            &CONFIRM_ACTIONS,
            request.strings,
        );
        let form = FormFrame {
            rect: area,
            rows: Vec::new(),
            buttons,
        };
        self.confirm = Some(ConfirmFrame {
            popup,
            message,
            form,
        });
    }
}
