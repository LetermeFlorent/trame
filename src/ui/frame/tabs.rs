use std::sync::Arc;

use super::label::fit_label;
use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::{TextLayout, TextSystem};
use crate::theme::Swatch;
use crate::ui::metrics::UiMetrics;
use crate::workspace::Pane;

const BUTTON_GAP: f32 = 2.0;
const BUTTON_MARGIN: f32 = 6.0;
const CLOSE_MARGIN: f32 = 5.0;
const LABEL_GAP: f32 = 6.0;

#[derive(Clone, Debug)]
pub struct TabFrame {
    pub id: SessionId,
    pub index: usize,
    pub rect: Rect,
    pub close: Rect,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
    pub active: bool,
    pub color: Option<Swatch>,
}

#[derive(Clone, Debug)]
pub struct TabStrip {
    pub rect: Rect,
    pub tabs_area: Rect,
    pub tabs: Vec<TabFrame>,
    pub new_tab: Rect,
    pub split: Rect,
}

impl TabStrip {
    pub fn insertion_index(&self, x: f32) -> usize {
        self.tabs
            .iter()
            .position(|tab| x < tab.rect.center().x)
            .unwrap_or(self.tabs.len())
    }
}

pub fn layout_strip(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    rect: Rect,
    pane: &Pane,
    title: &dyn Fn(SessionId) -> String,
) -> TabStrip {
    let button = metrics.px(UiMetrics::BUTTON);
    let margin = metrics.px(BUTTON_MARGIN);
    let button_y = (rect.y + (rect.height - button) / 2.0).round();
    let split = Rect::new(rect.right() - margin - button, button_y, button, button);
    let new_tab = Rect::new(
        split.x - metrics.px(BUTTON_GAP) - button,
        button_y,
        button,
        button,
    );
    let tabs_area = Rect::from_edges(rect.x, rect.y, new_tab.x - margin, rect.bottom());
    let tabs = layout_tabs(text, metrics, tabs_area, pane, title);
    TabStrip {
        rect,
        tabs_area,
        tabs,
        new_tab,
        split,
    }
}

fn layout_tabs(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    area: Rect,
    pane: &Pane,
    title: &dyn Fn(SessionId) -> String,
) -> Vec<TabFrame> {
    let style = metrics.label();
    let padding = metrics.px(UiMetrics::TAB_PADDING);
    let close = metrics.px(UiMetrics::TAB_CLOSE);
    let close_margin = metrics.px(CLOSE_MARGIN);
    let label_gap = metrics.px(LABEL_GAP);
    let fixed = padding + label_gap + close + close_margin;
    let minimum = metrics.px(UiMetrics::TAB_MIN);
    let maximum = metrics.px(UiMetrics::TAB_MAX);
    let titles: Vec<String> = pane.tabs.iter().map(|tab| title(*tab)).collect();
    let natural: Vec<f32> = titles
        .iter()
        .map(|label| {
            (text.layout(label, style).width + fixed)
                .ceil()
                .clamp(minimum, maximum)
        })
        .collect();
    let widths = fit_widths(&natural, area.width, minimum);
    let mut x = area.x - scroll_offset(&widths, pane.active, area.width);
    let mut frames = Vec::with_capacity(widths.len());
    for (index, (tab, width)) in pane.tabs.iter().zip(&widths).enumerate() {
        let rect = Rect::new(x, area.y, *width, area.height);
        let close_rect = Rect::new(
            rect.right() - close_margin - close,
            (rect.y + (rect.height - close) / 2.0).round(),
            close,
            close,
        );
        let label_x = rect.x + padding;
        let label_width = close_rect.x - label_gap - label_x;
        frames.push(TabFrame {
            id: *tab,
            index,
            rect,
            close: close_rect,
            label: fit_label(text, &titles[index], style, label_width),
            label_x,
            active: index == pane.active,
            color: None,
        });
        x += width;
    }
    frames
}

fn fit_widths(natural: &[f32], available: f32, minimum: f32) -> Vec<f32> {
    let total: f32 = natural.iter().sum();
    if total <= available || total <= 0.0 {
        return natural.to_vec();
    }
    let scale = available / total;
    natural
        .iter()
        .map(|width| (width * scale).floor().max(minimum))
        .collect()
}

fn scroll_offset(widths: &[f32], active: usize, available: f32) -> f32 {
    let end: f32 = widths.iter().take(active + 1).sum();
    (end - available).max(0.0)
}
