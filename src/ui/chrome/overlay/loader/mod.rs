mod marks;

use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::text::{TextLayout, TextSystem};
use crate::theme::Rgba;
use crate::ui::chrome::ChromeInput;
use crate::ui::chrome::logo::draw_logo;
use marks::{DOT_RADIUS, RING_RADIUS, draw_bar, draw_ring};

const LOGO: f32 = 72.0;
const LOGO_GAP: f32 = 30.0;
const MESSAGE_GAP: f32 = 26.0;
const BAR_GAP: f32 = 16.0;
const BAR_WIDTH: f32 = 220.0;
const BAR_HEIGHT: f32 = 4.0;
const LABEL_GAP: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoaderProgress<'a> {
    pub fraction: f32,
    pub label: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoaderView<'a> {
    pub message: &'a str,
    pub progress: Option<LoaderProgress<'a>>,
    pub spin: f32,
    pub opacity: f32,
}

pub fn draw_loader(
    scene: &mut Scene,
    text: &mut TextSystem,
    input: &ChromeInput,
    loader: &LoaderView,
) {
    if loader.opacity <= 0.0 {
        return;
    }
    let frame = input.frame;
    let (_, area) = frame.bounds.split_top(frame.title_bar.rect.height);
    scene.push_opacity(loader.opacity);
    scene.fill(area, input.theme.chrome, 0.0);
    draw_column(scene, text, input, loader, area);
    scene.pop_opacity();
}

fn draw_column(
    scene: &mut Scene,
    text: &mut TextSystem,
    input: &ChromeInput,
    loader: &LoaderView,
    area: Rect,
) {
    let metrics = input.frame.metrics;
    let theme = input.theme;
    let message = text.layout(loader.message, metrics.label());
    let logo = metrics.px(LOGO);
    let ring = metrics.px(RING_RADIUS) + metrics.px(DOT_RADIUS);
    let line = message.ascent + message.descent;
    let bar = metrics.px(BAR_HEIGHT);
    let spaced = metrics.px(LOGO_GAP) + metrics.px(MESSAGE_GAP) + metrics.px(BAR_GAP);
    let center = area.center().x;
    let mut y = (area.center().y - (logo + 2.0 * ring + line + bar + spaced) / 2.0).round();
    draw_logo(
        scene,
        Rect::new((center - logo / 2.0).round(), y, logo, logo),
    );
    y += logo + metrics.px(LOGO_GAP) + ring;
    draw_ring(
        scene,
        Point::new(center, y),
        metrics,
        theme.accent,
        loader.spin,
    );
    y += ring + metrics.px(MESSAGE_GAP);
    draw_centered(
        scene,
        text,
        &message,
        Point::new(center, y),
        theme.text_muted,
    );
    y += line + metrics.px(BAR_GAP);
    let width = metrics.px(BAR_WIDTH);
    let track = Rect::new((center - width / 2.0).round(), y.round(), width, bar);
    draw_bar(scene, track, theme, loader);
    let Some(progress) = loader.progress else {
        return;
    };
    let label = text.layout(progress.label, metrics.caption());
    let top = Point::new(center, track.bottom() + metrics.px(LABEL_GAP));
    draw_centered(scene, text, &label, top, theme.text_muted);
}

fn draw_centered(
    scene: &mut Scene,
    text: &mut TextSystem,
    layout: &TextLayout,
    top: Point,
    color: Rgba,
) {
    let pen = Point::new(
        (top.x - layout.width / 2.0).round(),
        (top.y + layout.ascent).round(),
    );
    text.draw_layout(scene, layout, pen, color);
}
