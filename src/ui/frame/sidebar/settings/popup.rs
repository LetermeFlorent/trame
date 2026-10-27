use crate::geometry::Rect;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;

use super::SettingLine;
use super::column::Column;

const PANEL_WIDTH: f32 = 360.0;
const PANEL_PAD: f32 = 12.0;
const TITLE_HEIGHT: f32 = 36.0;

#[derive(Clone, Debug)]
pub struct PopupFrame {
    pub backdrop: Rect,
    pub rect: Rect,
    pub title: SettingLine,
    pub close: Rect,
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

    pub(in crate::ui::frame) fn popup_room(&self) -> f32 {
        let metrics = &self.metrics;
        let chrome = 2.0 * metrics.px(PANEL_PAD) + metrics.px(TITLE_HEIGHT);
        (self.popup_backdrop().height - chrome).max(0.0)
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
        let width = metrics.px(wide).min(backdrop.width);
        let height = 2.0 * pad + metrics.px(TITLE_HEIGHT) + content;
        let x = backdrop.centered_x(width);
        let y = backdrop.centered_y(height).max(backdrop.y);
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
