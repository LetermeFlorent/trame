use std::borrow::Cow;
use std::sync::Arc;

use super::FrameInput;
use super::label::{fit_label, fit_label_start};
use crate::files::{home_dir, short_path};
use crate::geometry::Rect;
use crate::text::{TextLayout, TextSystem};
use crate::ui::metrics::{Platform, UiMetrics};

const TOGGLE_MARGIN: f32 = 8.0;
const TITLE_GAP: f32 = 10.0;
const SUBTITLE_GAP: f32 = 8.0;
const BRAND_GAP: f32 = 16.0;
const HEADING_MIN: f32 = 180.0;
const ROUND_BUTTON: f32 = 24.0;
const ROUND_GAP: f32 = 8.0;
const ROUND_MARGIN: f32 = 10.0;
const APP_NAME: &str = concat!("trame ", env!("CARGO_PKG_VERSION"));

type Placed = Option<(Arc<TextLayout>, f32)>;

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
    pub title: Placed,
    pub subtitle: Placed,
    pub brand: Placed,
    pub controls: Option<WindowControls>,
    pub maximized: bool,
}

pub fn layout_title_bar(text: &mut TextSystem, input: &FrameInput, rect: Rect) -> TitleBarFrame {
    let metrics = &input.metrics;
    let toggle = sidebar_toggle(metrics, rect, input.fullscreen);
    let controls = metrics
        .platform
        .draws_window_controls()
        .then(|| window_controls(metrics, rect));
    let start = toggle.right() + metrics.px(TITLE_GAP);
    let end = controls.map_or(rect.right(), |controls| controls.minimize.x) - metrics.px(TITLE_GAP);
    let brand = layout_brand(text, input, rect, start, end);
    let limit = brand
        .as_ref()
        .map_or(end, |(_, x)| x - metrics.px(BRAND_GAP));
    let (title, subtitle) = layout_heading(text, input, start, limit);
    TitleBarFrame {
        rect,
        toggle,
        title,
        subtitle,
        brand,
        controls,
        maximized: input.maximized,
    }
}

fn layout_brand(
    text: &mut TextSystem,
    input: &FrameInput,
    rect: Rect,
    start: f32,
    end: f32,
) -> Placed {
    let metrics = &input.metrics;
    let name: Cow<str> = match input.unsaved {
        true => format!("{APP_NAME} ({})", input.strings.unsaved_session).into(),
        false => APP_NAME.into(),
    };
    let layout = text.layout(&name, metrics.label());
    let x = rect.centered_x(layout.width).round();
    let room = x - metrics.px(BRAND_GAP) - start;
    let fits = room >= metrics.px(HEADING_MIN) && x + layout.width <= end;
    fits.then_some((layout, x))
}

fn layout_heading(
    text: &mut TextSystem,
    input: &FrameInput,
    start: f32,
    limit: f32,
) -> (Placed, Placed) {
    let metrics = &input.metrics;
    let name = input
        .workspace
        .active_instance()
        .map(|instance| fit_label(text, &instance.name, metrics.strong(), limit - start));
    let folder_x = name
        .as_ref()
        .map_or(start, |name| start + name.width + metrics.px(SUBTITLE_GAP));
    let folder = input
        .folder
        .filter(|_| limit - folder_x > metrics.px(UiMetrics::TAB_MIN))
        .map(|folder| {
            let shown = short_path(folder, home_dir().as_deref());
            fit_label_start(text, &shown, metrics.label(), limit - folder_x)
        });
    (
        name.map(|layout| (layout, start)),
        folder.map(|layout| (layout, folder_x)),
    )
}

fn sidebar_toggle(metrics: &UiMetrics, rect: Rect, fullscreen: bool) -> Rect {
    let inset = match metrics.platform {
        Platform::Mac if !fullscreen => metrics.px(UiMetrics::MAC_CONTROLS_INSET),
        _ => 0.0,
    };
    let button = metrics.px(UiMetrics::BUTTON);
    rect.square_at(rect.x + inset + metrics.px(TOGGLE_MARGIN), button)
}

fn window_controls(metrics: &UiMetrics, rect: Rect) -> WindowControls {
    let (width, height, gap, margin) = match metrics.platform {
        Platform::Windows => (metrics.px(UiMetrics::WINDOW_BUTTON), rect.height, 0.0, 0.0),
        _ => {
            let size = metrics.px(ROUND_BUTTON);
            (size, size, metrics.px(ROUND_GAP), metrics.px(ROUND_MARGIN))
        }
    };
    let y = rect.centered_y(height).round();
    let close = Rect::new(rect.right() - margin - width, y, width, height);
    let maximize = Rect::new(close.x - gap - width, y, width, height);
    let minimize = Rect::new(maximize.x - gap - width, y, width, height);
    WindowControls {
        minimize,
        maximize,
        close,
    }
}
