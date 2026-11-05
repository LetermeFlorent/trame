use std::sync::Arc;

use crate::geometry::Rect;
use crate::text::{TextLayout, TextSystem};
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;

const ICON_GAP: f32 = 4.0;
const LABEL_MIN: f32 = 24.0;
const AREA_SHARE: f32 = 0.5;

#[derive(Clone, Debug)]
pub struct HostBadge {
    pub rect: Rect,
    pub icon: Rect,
    pub label: Option<Arc<TextLayout>>,
    pub label_x: f32,
}

pub(super) struct HostText {
    pub(super) label: String,
    pub(super) width: f32,
}

impl HostText {
    pub(super) fn measure(text: &mut TextSystem, metrics: &UiMetrics, label: String) -> Self {
        let width = text.layout(&label, metrics.label()).width;
        Self { label, width }
    }
}

pub(super) fn width(metrics: &UiMetrics, host: &HostText, space: f32) -> f32 {
    let icon = metrics.px(UiMetrics::ICON);
    let gap = metrics.px(ICON_GAP);
    let room = (host.width + icon + gap).min(space * AREA_SHARE);
    match room - icon - gap >= metrics.px(LABEL_MIN) {
        true => room.ceil(),
        false => icon,
    }
}

pub(super) fn place(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    host: &HostText,
    rect: Rect,
) -> HostBadge {
    let size = metrics.px(UiMetrics::ICON);
    let icon = rect.square_at(rect.x, size);
    let label_x = icon.right() + metrics.px(ICON_GAP);
    let room = rect.right() - label_x;
    let label = (room > 0.0).then(|| fit_label(text, &host.label, metrics.label(), room));
    HostBadge {
        rect,
        icon,
        label,
        label_x,
    }
}
