use crate::geometry::Point;
use crate::locale::Strings;
use crate::text::TextSystem;
use crate::ui::frame::editor::{FormAction, FormFrame, button_form};
use crate::ui::frame::{FrameLayout, fit_label_start};
use crate::ui::metrics::UiMetrics;

use super::column::{LINE_HEIGHT, LINE_INSET, SECTION_GAP};
use super::{PopupFrame, SettingLine};

#[derive(Clone, Debug)]
pub struct MessageFrame {
    pub popup: PopupFrame,
    pub lines: Vec<SettingLine>,
    pub form: FormFrame,
}

pub struct MessageRequest<'a> {
    pub title: &'a str,
    pub lines: &'a [&'a str],
    pub actions: &'a [FormAction],
    pub strings: &'static Strings,
}

impl MessageFrame {
    pub fn button_at(&self, point: Point) -> Option<FormAction> {
        self.form.button_at(point)
    }
}

impl FrameLayout {
    pub fn place_message(&mut self, text: &mut TextSystem, request: &MessageRequest) {
        let metrics = self.metrics;
        let actions = metrics.px(SECTION_GAP) + metrics.px(UiMetrics::BUTTON);
        let content = request.lines.len() as f32 * metrics.px(LINE_HEIGHT) + actions;
        let (popup, mut column) = self.open_popup(text, request.title, content);
        let mut lines = Vec::with_capacity(request.lines.len());
        for shown in request.lines {
            let mut line = column.line("", LINE_HEIGHT, metrics.label());
            let room = line.rect.width - 2.0 * metrics.px(LINE_INSET);
            line.label = fit_label_start(column.text, shown, metrics.label(), room);
            lines.push(line);
        }
        let area = column.band(actions);
        let form = button_form(
            column.text,
            &metrics,
            area,
            request.actions,
            request.strings,
        );
        self.message = Some(MessageFrame { popup, lines, form });
    }
}
