use std::sync::Arc;

use super::label::fit_label;
use crate::geometry::Rect;
use crate::session::SessionId;
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::{Platform, UiMetrics};
use crate::workspace::Workspace;

const TOGGLE_MARGIN: f32 = 8.0;
const TITLE_GAP: f32 = 10.0;
const SUBTITLE_GAP: f32 = 8.0;
const ROUND_BUTTON: f32 = 24.0;
const ROUND_GAP: f32 = 8.0;
const ROUND_MARGIN: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowControls {
    pub minimize: Rect,
    pub maximize: Rect,
    pub close: Rect,
}

#[derive(Clone, Debug)]
pub struct TitleBarFrame {
    pub rect: Rect,
    pub toggle: Rect,
    pub title: Option<(Arc<TextLayout>, f32)>,
    pub subtitle: Option<(Arc<TextLayout>, f32)>,
    pub controls: Option<WindowControls>,
    pub maximized: bool,
}

pub struct TitleInput<'a> {
    pub workspace: &'a Workspace,
    pub maximized: bool,
    pub title: &'a dyn Fn(SessionId) -> String,
}

pub fn layout_title_bar(
    text: &mut TextSystem,
    metrics: &UiMetrics,
    rect: Rect,
    input: TitleInput,
) -> TitleBarFrame {
    let inset = match metrics.platform {
        Platform::Mac => metrics.px(UiMetrics::MAC_CONTROLS_INSET),
        _ => 0.0,
    };
    let button = metrics.px(UiMetrics::BUTTON);
    let toggle = Rect::new(
        rect.x + inset + metrics.px(TOGGLE_MARGIN),
        (rect.y + (rect.height - button) / 2.0).round(),
        button,
        button,
    );
    let controls = metrics
        .platform
        .draws_window_controls()
        .then(|| window_controls(metrics, rect));
    let limit =
        controls.map_or(rect.right(), |controls| controls.minimize.x) - metrics.px(TITLE_GAP);
    let title_x = toggle.right() + metrics.px(TITLE_GAP);
    let name = input
        .workspace
        .active_instance()
        .map(|instance| fit_label(text, &instance.name, metrics.strong(), limit - title_x));
    let subtitle_x = name
        .as_ref()
        .map_or(title_x, |name| title_x + name.width + metrics.px(SUBTITLE_GAP));
    let subtitle = input
        .workspace
        .active_tab()
        .filter(|_| limit - subtitle_x > metrics.px(UiMetrics::TAB_MIN))
        .map(|tab| fit_label(text, &(input.title)(tab), metrics.label(), limit - subtitle_x));
    TitleBarFrame {
        rect,
        toggle,
        title: name.map(|layout| (layout, title_x)),
        subtitle: subtitle.map(|layout| (layout, subtitle_x)),
        controls,
        maximized: input.maximized,
    }
}

fn window_controls(metrics: &UiMetrics, rect: Rect) -> WindowControls {
    let (width, height, gap, margin) = match metrics.platform {
        Platform::Windows => (metrics.px(UiMetrics::WINDOW_BUTTON), rect.height, 0.0, 0.0),
        _ => {
            let size = metrics.px(ROUND_BUTTON);
            (size, size, metrics.px(ROUND_GAP), metrics.px(ROUND_MARGIN))
        }
    };
    let y = (rect.y + (rect.height - height) / 2.0).round();
    let close = Rect::new(rect.right() - margin - width, y, width, height);
    let maximize = Rect::new(close.x - gap - width, y, width, height);
    let minimize = Rect::new(maximize.x - gap - width, y, width, height);
    WindowControls {
        minimize,
        maximize,
        close,
    }
}
