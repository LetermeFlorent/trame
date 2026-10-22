use std::f32::consts::TAU;

use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::text::TextSystem;
use crate::theme::Rgba;
use crate::ui::chrome::ChromeInput;
use crate::ui::chrome::logo::draw_logo;

const LOGO: f32 = 72.0;
const LOGO_GAP: f32 = 30.0;
const RING_RADIUS: f32 = 11.0;
const DOT_RADIUS: f32 = 2.25;
const DOTS: usize = 10;
const TURN_SECONDS: f32 = 1.1;
const FAINTEST_DOT: f32 = 0.12;
const MESSAGE_GAP: f32 = 26.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoaderView<'a> {
    pub message: &'a str,
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
    let metrics = frame.metrics;
    let theme = input.theme;
    let (_, area) = frame.bounds.split_top(frame.title_bar.rect.height);
    let message = text.layout(loader.message, metrics.label());
    let logo = metrics.px(LOGO).round();
    let ring = metrics.px(RING_RADIUS);
    let dot = metrics.px(DOT_RADIUS);
    let line = message.ascent + message.descent;
    let logo_gap = metrics.px(LOGO_GAP);
    let message_gap = metrics.px(MESSAGE_GAP);
    let height = logo + logo_gap + 2.0 * (ring + dot) + message_gap + line;
    let center = area.center().x;
    let top = (area.center().y - height / 2.0).round();
    scene.push_opacity(loader.opacity);
    scene.fill(area, theme.chrome, 0.0);
    draw_logo(
        scene,
        Rect::new((center - logo / 2.0).round(), top, logo, logo),
    );
    let ring_center = Point::new(center, top + logo + logo_gap + ring + dot);
    let shape = Ring {
        center: ring_center,
        radius: ring,
        dot,
    };
    draw_ring(scene, &shape, theme.accent, loader.spin);
    let baseline = (ring_center.y + ring + dot + message_gap + message.ascent).round();
    let pen = Point::new((center - message.width / 2.0).round(), baseline);
    text.draw_layout(scene, &message, pen, theme.text_muted);
    scene.pop_opacity();
}

struct Ring {
    center: Point,
    radius: f32,
    dot: f32,
}

fn draw_ring(scene: &mut Scene, ring: &Ring, color: Rgba, spin: f32) {
    let Ring {
        center,
        radius,
        dot,
    } = *ring;
    let head = (spin / TURN_SECONDS).rem_euclid(1.0) * TAU;
    for index in 0..DOTS {
        let behind = index as f32 / DOTS as f32;
        let angle = head - behind * TAU;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        let alpha = 1.0 - behind * (1.0 - FAINTEST_DOT);
        let rect = Rect::new(x - dot, y - dot, 2.0 * dot, 2.0 * dot);
        scene.fill(rect, color.with_alpha(color.a * alpha), dot);
    }
}
