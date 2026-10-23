use std::sync::Arc;

use super::PaneContext;
use super::widths::{TabSizing, fit_widths, scroll_offset};
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
    let button_y = rect.centered_y(button).round();
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
    let sizing = TabSizing::new(&cx.metrics);
    let pane = placement.pane;
    let titles: Vec<String> = pane.tabs.iter().map(|tab| (cx.title)(*tab)).collect();
    let natural: Vec<f32> = titles
        .iter()
        .map(|title| sizing.natural(cx.text, title))
        .collect();
    let widths = fit_widths(&natural, available, sizing.minimum);
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
        frames.push(TabFrame {
            id: *tab,
            index,
            rect,
            close: sizing.close_rect(rect),
            label: fit_label(cx.text, &titles[index], sizing.style, width - sizing.fixed),
            label_x: rect.x + sizing.padding,
            active: index == pane.active,
            color: None,
        });
        x += width;
    }
    frames
}
