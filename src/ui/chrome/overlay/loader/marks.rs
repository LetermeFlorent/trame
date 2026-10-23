use std::f32::consts::TAU;

use super::LoaderView;
use crate::geometry::{Point, Rect};
use crate::render::Scene;
use crate::theme::{Rgba, Theme};
use crate::ui::UiMetrics;

pub(super) const RING_RADIUS: f32 = 11.0;
pub(super) const DOT_RADIUS: f32 = 2.25;
const DOTS: usize = 10;
const TURN_SECONDS: f32 = 1.1;
const FAINTEST_DOT: f32 = 0.12;
const SWEEP_SECONDS: f32 = 1.4;
const SWEEP_SHARE: f32 = 0.3;
const TRACK_ALPHA: f32 = 0.2;

pub(super) fn draw_ring(
    scene: &mut Scene,
    center: Point,
    metrics: UiMetrics,
    color: Rgba,
    spin: f32,
) {
    let radius = metrics.px(RING_RADIUS);
    let dot = metrics.px(DOT_RADIUS);
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

pub(super) fn draw_bar(scene: &mut Scene, track: Rect, theme: &Theme, loader: &LoaderView) {
    let radius = track.height / 2.0;
    scene.fill(track, theme.text_muted.with_alpha(TRACK_ALPHA), radius);
    let (start, share) = match loader.progress {
        Some(progress) => (0.0, progress.fraction.clamp(0.0, 1.0)),
        None => {
            let sweep = (loader.spin / SWEEP_SECONDS).rem_euclid(1.0);
            (sweep * (1.0 + SWEEP_SHARE) - SWEEP_SHARE, SWEEP_SHARE)
        }
    };
    let width = track.width;
    let filled = Rect::new(
        track.x + start * width,
        track.y,
        share * width,
        track.height,
    );
    scene.fill(filled.intersect(track), theme.accent, radius);
}
