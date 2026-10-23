use std::sync::Arc;

use super::clamp_x;
use crate::geometry::{Point, Rect};
use crate::text::{TextLayout, TextSystem};
use crate::ui::frame::FrameLayout;
use crate::ui::metrics::UiMetrics;

const MENU_PAD: f32 = 4.0;
const ROW_PAD_X: f32 = 10.0;
const MIN_WIDTH: f32 = 148.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEntry {
    Rename,
    Color,
    Icon,
    Folder,
    Ssh,
}

pub struct MenuRequest<'a> {
    pub at: Point,
    pub entries: &'a [(MenuEntry, &'a str)],
}

#[derive(Clone, Debug)]
pub struct MenuRow {
    pub rect: Rect,
    pub entry: MenuEntry,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
}

#[derive(Clone, Debug)]
pub struct MenuFrame {
    pub rect: Rect,
    pub rows: Vec<MenuRow>,
}

impl MenuFrame {
    pub fn entry_at(&self, point: Point) -> Option<MenuEntry> {
        self.rows
            .iter()
            .find(|row| row.rect.contains(point))
            .map(|row| row.entry)
    }
}

impl FrameLayout {
    pub fn place_menu(&mut self, text: &mut TextSystem, request: &MenuRequest) {
        let metrics = &self.metrics;
        let pad = metrics.px(MENU_PAD);
        let row_pad = metrics.px(ROW_PAD_X);
        let row_height = metrics.px(UiMetrics::SIDEBAR_ROW);
        let labels: Vec<_> = request
            .entries
            .iter()
            .map(|(entry, label)| (*entry, text.layout(label, metrics.label())))
            .collect();
        let widest = labels
            .iter()
            .map(|(_, label)| label.width)
            .fold(0.0, f32::max);
        let width = (widest + 2.0 * row_pad).max(metrics.px(MIN_WIDTH)) + 2.0 * pad;
        let height = labels.len() as f32 * row_height + 2.0 * pad;
        let rect = menu_rect(request.at, width, height, self.bounds);
        let inner = rect.inset(pad);
        let rows = labels
            .into_iter()
            .enumerate()
            .map(|(index, (entry, label))| MenuRow {
                rect: Rect::new(
                    inner.x,
                    inner.y + index as f32 * row_height,
                    inner.width,
                    row_height,
                ),
                entry,
                label,
                label_x: inner.x + row_pad,
            })
            .collect();
        self.menu = Some(MenuFrame { rect, rows });
    }
}

fn menu_rect(at: Point, width: f32, height: f32, bounds: Rect) -> Rect {
    let y = at.y.min(bounds.bottom() - height).max(bounds.y).round();
    Rect::new(clamp_x(at.x, width, bounds), y, width, height)
}
