use crate::geometry::Rect;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::metrics::UiMetrics;

use super::SettingLine;
use super::column::Column;
use super::form::page_height;
use super::place::tabs_height;

const PANEL_WIDTH: f32 = 360.0;
const PANEL_PAD: f32 = 12.0;
const TITLE_HEIGHT: f32 = 36.0;
const POPUP_GAP: f32 = 16.0;

#[derive(Clone, Debug)]
pub struct PopupFrame {
    pub backdrop: Rect,
    pub rect: Rect,
    pub title: SettingLine,
    pub close: Rect,
}

impl PopupFrame {
    pub(in crate::ui::frame) fn end_below(&mut self, metrics: &UiMetrics, content: f32) {
        let bottom = content + metrics.px(PANEL_PAD);
        self.rect.height = (bottom - self.rect.y).min(self.rect.height).round();
    }
}

impl FrameLayout {
    fn popup_backdrop(&self) -> Rect {
        let bounds = self.bounds;
        Rect::from_edges(
            bounds.x,
            self.title_bar.rect.bottom(),
            bounds.right(),
            bounds.bottom(),
        )
    }

    fn popup_area(&self) -> Rect {
        self.popup_backdrop().inset(self.metrics.px(POPUP_GAP))
    }

    pub(in crate::ui::frame) fn popup_room(&self) -> f32 {
        (self.popup_area().height - self.popup_chrome()).max(0.0)
    }

    fn popup_top(&self, height: f32) -> f32 {
        let metrics = &self.metrics;
        let area = self.popup_area();
        let page = page_height(metrics, self.popup_room() - tabs_height(metrics));
        let settings = self.popup_chrome() + tabs_height(metrics) + page;
        let top = area.centered_y(settings).max(area.y);
        top.min(area.bottom() - height).max(area.y)
    }

    fn popup_chrome(&self) -> f32 {
        2.0 * self.metrics.px(PANEL_PAD) + self.metrics.px(TITLE_HEIGHT)
    }

    pub(in crate::ui::frame) fn open_popup<'a>(
        &self,
        text: &'a mut TextSystem,
        title: &str,
        content: f32,
    ) -> (PopupFrame, Column<'a>) {
        self.open_wide_popup(text, title, [PANEL_WIDTH, content])
    }

    pub(in crate::ui::frame) fn open_wide_popup<'a>(
        &self,
        text: &'a mut TextSystem,
        title: &str,
        [wide, content]: [f32; 2],
    ) -> (PopupFrame, Column<'a>) {
        let metrics = self.metrics;
        let backdrop = self.popup_backdrop();
        let pad = metrics.px(PANEL_PAD);
        let width = metrics.px(wide).min(self.popup_area().width);
        let height = 2.0 * pad + metrics.px(TITLE_HEIGHT) + content;
        let x = backdrop.centered_x(width);
        let y = self.popup_top(height);
        let rect = Rect::new(x, y, width, height).snapped();
        let mut column = Column {
            text,
            metrics,
            inner: rect.inset(pad),
            y: rect.y + pad,
        };
        let title = column.line(title, TITLE_HEIGHT, metrics.strong());
        let close = column.end_button(&title);
        let popup = PopupFrame {
            backdrop,
            rect,
            title,
            close,
        };
        (popup, column)
    }
}
