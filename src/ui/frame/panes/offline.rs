use std::sync::Arc;

use crate::geometry::Rect;
use crate::locale::Strings;
use crate::session::SessionId;
use crate::text::{TextLayout, TextSystem};
use crate::ui::frame::FrameLayout;
use crate::ui::metrics::UiMetrics;

const BUTTON_PAD_X: f32 = 12.0;
const GAP: f32 = 12.0;

#[derive(Clone, Debug)]
pub struct OfflineCard {
    pub tab: SessionId,
    pub line: Rect,
    pub message: Arc<TextLayout>,
    pub button: Rect,
    pub label: Arc<TextLayout>,
}

impl FrameLayout {
    pub fn mark_offline(
        &mut self,
        text: &mut TextSystem,
        strings: &'static Strings,
        offline: impl Fn(SessionId) -> bool,
    ) {
        let metrics = self.metrics;
        for pane in &mut self.panes {
            pane.offline = pane
                .active
                .filter(|tab| offline(*tab))
                .map(|tab| card(text, &metrics, strings, pane.body, tab));
        }
    }
}

fn card(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    strings: &'static Strings,
    body: Rect,
    tab: SessionId,
) -> OfflineCard {
    let message = text.layout(strings.waiting_connection, metrics.label());
    let label = text.layout(strings.disconnect, metrics.label());
    let height = metrics.px(UiMetrics::BUTTON);
    let gap = metrics.px(GAP);
    let top = (body.center().y - height - gap / 2.0).round();
    let width = label.width.ceil() + 2.0 * metrics.px(BUTTON_PAD_X);
    let x = body.centered_x(width).round();
    OfflineCard {
        tab,
        line: Rect::new(body.x, top, body.width, height),
        message,
        button: Rect::new(x, top + height + gap, width, height),
        label,
    }
}
