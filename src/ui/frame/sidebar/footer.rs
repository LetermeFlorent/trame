use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::text::{TextLayout, TextSystem};
use crate::ui::frame::FrameInput;
use crate::ui::hit::Hit;
use crate::ui::metrics::UiMetrics;

const LABEL_GAP: f32 = 2.0;
const STACK_GAP: f32 = 4.0;

#[derive(Clone, Debug)]
pub struct FooterButton {
    pub rect: Rect,
    pub icon: Rect,
    pub label: Arc<TextLayout>,
    pub label_x: f32,
}

#[derive(Clone, Debug)]
pub struct FooterFrame {
    pub rect: Rect,
    pub settings: FooterButton,
    pub update: Option<FooterButton>,
}

impl FooterFrame {
    pub fn above(&self, rect: Rect) -> Rect {
        Rect::from_edges(rect.x, rect.y, rect.right(), self.rect.y.max(rect.y))
    }

    pub(in crate::ui::frame) fn hit(&self, point: Point) -> Hit {
        if self.settings.rect.contains(point) {
            return Hit::Settings;
        }
        match &self.update {
            Some(update) if update.rect.contains(point) => Hit::Update,
            _ => Hit::Sidebar,
        }
    }
}

pub(crate) fn layout_footer(text: &mut TextSystem, input: &FrameInput, side: Rect) -> FooterFrame {
    let metrics = &input.metrics;
    let line = metrics.line();
    let size = metrics.px(UiMetrics::BUTTON);
    let stack = metrics.px(STACK_GAP);
    let gap = ((metrics.px(UiMetrics::SIDEBAR_HEADER) - line - size) / 2.0)
        .floor()
        .max(0.0);
    let extra = input.update.map_or(0.0, |_| size + stack);
    let shown = side.height - metrics.hidden_bottom(input.maximized);
    let height = (line + size + extra + 2.0 * gap).min(shown);
    let rect = Rect::new(side.x, side.y + shown - height, side.width, height);
    let inset = (metrics.px(UiMetrics::RAIL_WIDTH) - size) / 2.0;
    let width = (side.width - 2.0 * inset).max(size);
    let place = |y: f32| Rect::new(side.x + inset, y, width, size).snapped();
    let settings = place(rect.bottom() - gap - size);
    let update = input.update.map(|version| {
        let label = format!("{} {version}", input.strings.update_to);
        button(text, metrics, place(settings.y - stack - size), &label)
    });
    FooterFrame {
        rect,
        settings: button(text, metrics, settings, input.strings.settings),
        update,
    }
}

fn button(text: &mut TextSystem, metrics: &UiMetrics, rect: Rect, label: &str) -> FooterButton {
    let icon = Rect::new(rect.x, rect.y, rect.height, rect.height);
    FooterButton {
        rect,
        icon,
        label: text.layout(label, metrics.label()),
        label_x: icon.right() + metrics.px(LABEL_GAP),
    }
}
