use std::sync::Arc;

use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::text::{TextLayout, TextSystem};
use crate::theme::Theme;
use crate::ui::frame::fit_label;
use crate::ui::metrics::UiMetrics;

const GHOST_ALPHA: f32 = 0.85;
const GHOST_OFFSET: f32 = 10.0;
const GHOST_BLUR: f32 = 8.0;
const OPAQUE: u8 = 255;
const ARROW: [&str; 19] = [
    "B...........",
    "BB..........",
    "BWB.........",
    "BWWB........",
    "BWWWB.......",
    "BWWWWB......",
    "BWWWWWB.....",
    "BWWWWWWB....",
    "BWWWWWWWB...",
    "BWWWWWWWWB..",
    "BWWWWWWWWWB.",
    "BWWWWWWBBBBB",
    "BWWWBWWB....",
    "BWWBBWWB....",
    "BWB..BWWB...",
    "BB...BWWB...",
    "B.....BWWB..",
    "......BWWB..",
    ".......BB...",
];

pub(super) struct Ghost<'a> {
    pub label: &'a str,
    pub pointer: Point,
}

fn fitted_label(text: &mut TextSystem, metrics: &UiMetrics, label: &str) -> (Arc<TextLayout>, f32) {
    let padding = metrics.px(UiMetrics::TAB_PADDING);
    let widest = metrics.px(UiMetrics::TAB_MAX) - padding * 2.0;
    (fit_label(text, label, metrics.label(), widest), padding)
}

pub(super) fn paint_ghost(
    scene: &mut Scene,
    text: &mut TextSystem,
    theme: &Theme,
    metrics: UiMetrics,
    ghost: Ghost,
) {
    let (label, padding) = fitted_label(text, &metrics, ghost.label);
    let offset = metrics.px(GHOST_OFFSET);
    let rect = Rect::new(
        (ghost.pointer.x + offset).round(),
        (ghost.pointer.y + offset).round(),
        (label.width + padding * 2.0).ceil(),
        metrics.spaced(UiMetrics::TAB_BAR),
    );
    let radius = metrics.px(UiMetrics::RADIUS);
    scene.shadow(rect, theme.shadow, radius, metrics.px(GHOST_BLUR));
    scene.fill(rect, theme.terminal.with_alpha(GHOST_ALPHA), radius);
    scene.border(rect, theme.border_strong, radius, metrics.line());
    let pen = Point::new((rect.x + padding).round(), label.baseline_in(rect));
    text.draw_layout(scene, &label, pen, theme.text);
}

pub fn ghost_scene(text: &mut TextSystem, theme: &Theme, metrics: UiMetrics, label: &str) -> Scene {
    let (fitted, padding) = fitted_label(text, &metrics, label);
    let width = fitted.width;
    let margin = metrics.px(GHOST_OFFSET) + 2.0 * metrics.px(GHOST_BLUR);
    let arrow = arrow_scale(metrics);
    let mut scene = Scene::new(
        (width + padding * 2.0 + margin)
            .ceil()
            .max(arrow_width(arrow)),
        (metrics.spaced(UiMetrics::TAB_BAR) + margin)
            .ceil()
            .max(arrow_height(arrow)),
    );
    let ghost = Ghost {
        label,
        pointer: Point::new(0.0, 0.0),
    };
    paint_ghost(&mut scene, text, theme, metrics, ghost);
    scene
}

pub fn straight_alpha(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 {
            pixel[..3].fill(0);
            continue;
        }
        for channel in &mut pixel[..3] {
            *channel = (u32::from(*channel) * 255 / alpha).min(255) as u8;
        }
    }
}

pub fn stamp_arrow(pixels: &mut [u8], width: u32, metrics: UiMetrics) {
    let scale = arrow_scale(metrics) as usize;
    let width = width as usize;
    for (row, line) in ARROW.iter().enumerate() {
        for (col, mark) in line.bytes().enumerate() {
            let shade = match mark {
                b'B' => 0,
                b'W' => OPAQUE,
                _ => continue,
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    let x = col * scale + dx;
                    let at = ((row * scale + dy) * width + x) * 4;
                    if x < width && at + 4 <= pixels.len() {
                        pixels[at..at + 4].copy_from_slice(&[shade, shade, shade, OPAQUE]);
                    }
                }
            }
        }
    }
}

fn arrow_scale(metrics: UiMetrics) -> f32 {
    metrics.scale.round().max(1.0)
}

fn arrow_width(scale: f32) -> f32 {
    ARROW[0].len() as f32 * scale
}

fn arrow_height(scale: f32) -> f32 {
    ARROW.len() as f32 * scale
}
