use std::sync::Arc;

use super::PaneContext;
use super::widths::{fit_widths, scroll_offset};
use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::TextLayout;
use crate::theme::Swatch;
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;
use crate::ui::motion::MotionKey;
use crate::workspace::{Pane, PaneId};

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

pub(super) struct StripPlacement<'a> {
    pub(super) pane_id: PaneId,
    pub(super) pane: &'a Pane,
    pub(super) bar: Rect,
    pub(super) target: Rect,
    pub(super) grows: bool,
}

struct Controls {
    tabs_area: Rect,
    new_tab: Rect,
    split: Rect,
}

pub(super) fn layout_strip(cx: &mut PaneContext, placement: StripPlacement) -> TabStrip {
    let controls = controls(&cx.metrics, placement.bar);
    let target_area = self::controls(&cx.metrics, placement.target).tabs_area;
    let tabs = layout_tabs(cx, &placement, controls.tabs_area, target_area.width);
    TabStrip {
        rect: placement.bar,
        tabs_area: controls.tabs_area,
        tabs,
        new_tab: controls.new_tab,
        split: controls.split,
    }
}

fn controls(metrics: &UiMetrics, rect: Rect) -> Controls {
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
    Controls {
        tabs_area,
        new_tab,
        split,
    }
}

fn layout_tabs(
    cx: &mut PaneContext,
    placement: &StripPlacement,
    area: Rect,
    available: f32,
) -> Vec<TabFrame> {
    let metrics = cx.metrics;
    let style = metrics.label();
    let padding = metrics.px(UiMetrics::TAB_PADDING);
    let close = metrics.px(UiMetrics::TAB_CLOSE);
    let close_margin = metrics.px(CLOSE_MARGIN);
    let fixed = padding + metrics.px(LABEL_GAP) + close + close_margin;
    let minimum = metrics.px(UiMetrics::TAB_MIN);
    let maximum = metrics.px(UiMetrics::TAB_MAX);
    let pane = placement.pane;
    let titles: Vec<String> = pane.tabs.iter().map(|tab| (cx.title)(*tab)).collect();
    let natural: Vec<f32> = titles
        .iter()
        .map(|label| {
            (cx.text.layout(label, style).width + fixed)
                .ceil()
                .clamp(minimum, maximum)
        })
        .collect();
    let widths = fit_widths(&natural, available, minimum);
    let mut x = -scroll_offset(&widths, pane.active, available);
    let mut frames = Vec::with_capacity(widths.len());
    for (index, (tab, width)) in pane.tabs.iter().zip(&widths).enumerate() {
        let target = Rect::new(x, 0.0, *width, area.height);
        let entry = placement
            .grows
            .then_some(Rect::new(x, 0.0, 0.0, area.height));
        let key = MotionKey::Tab(placement.pane_id, *tab);
        let rect = cx
            .motion
            .place(key, target, entry)
            .translated(area.x, area.y);
        let close_rect = Rect::new(
            rect.right() - close_margin - close,
            (rect.y + (rect.height - close) / 2.0).round(),
            close,
            close,
        );
        frames.push(TabFrame {
            id: *tab,
            index,
            rect,
            close: close_rect,
            label: fit_label(cx.text, &titles[index], style, width - fixed),
            label_x: rect.x + padding,
            active: index == pane.active,
            color: None,
        });
        x += width;
    }
    frames
}
