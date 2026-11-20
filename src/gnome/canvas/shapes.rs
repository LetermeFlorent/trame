use std::f64::consts::{FRAC_PI_2, PI};

use gtk::cairo;
use trame::render::Quad;

const SHADOW_SPREAD: f32 = 2.0;
const SHADOW_STEPS: u32 = 4;
const SHADOW_STEP_ALPHA: f32 = 0.22;

pub fn set_rgba(cr: &cairo::Context, color: [f32; 4], alpha_scale: f32) {
    cr.set_source_rgba(
        f64::from(color[0]),
        f64::from(color[1]),
        f64::from(color[2]),
        f64::from((color[3] * alpha_scale).clamp(0.0, 1.0)),
    );
}

pub fn clip_reach(cr: &cairo::Context, clip: [f32; 4], reach: [f32; 4]) -> bool {
    let left = reach[0].max(clip[0]);
    let top = reach[1].max(clip[1]);
    let right = (reach[0] + reach[2]).min(clip[2]);
    let bottom = (reach[1] + reach[3]).min(clip[3]);
    if right <= left || bottom <= top {
        return false;
    }
    cr.rectangle(
        f64::from(left),
        f64::from(top),
        f64::from(right - left),
        f64::from(bottom - top),
    );
    cr.clip();
    true
}

pub fn shadow_reach(quad: &Quad) -> [f32; 4] {
    let grow = quad.params[2].max(0.0) * SHADOW_SPREAD;
    [
        quad.rect[0] - grow,
        quad.rect[1] - grow,
        quad.rect[2] + grow * 2.0,
        quad.rect[3] + grow * 2.0,
    ]
}

pub fn rounded_rect(cr: &cairo::Context, rect: [f32; 4], radius: f32) {
    let [x, y, w, h] = rect.map(f64::from);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let limit = (w * 0.5).min(h * 0.5);
    let r = f64::from(radius).clamp(0.0, limit);
    if r <= 0.0 {
        cr.rectangle(x, y, w, h);
        return;
    }
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    cr.arc(x + r, y + r, r, PI, PI + FRAC_PI_2);
    cr.close_path();
}

pub fn fill_rounded(cr: &cairo::Context, quad: &Quad) {
    cr.new_path();
    rounded_rect(cr, quad.rect, quad.params[1]);
    set_rgba(cr, quad.color, 1.0);
    let _ = cr.fill();
}

pub fn stroke_border(cr: &cairo::Context, quad: &Quad) {
    let width = quad.params[2].max(0.0);
    if width <= 0.0 {
        return;
    }
    let radius = quad.params[1];
    cr.new_path();
    rounded_rect(cr, quad.rect, radius);
    let inner = [
        quad.rect[0] + width,
        quad.rect[1] + width,
        quad.rect[2] - width * 2.0,
        quad.rect[3] - width * 2.0,
    ];
    if inner[2] > 0.0 && inner[3] > 0.0 {
        rounded_rect(cr, inner, (radius - width).max(0.0));
    }
    set_rgba(cr, quad.color, 1.0);
    cr.set_fill_rule(cairo::FillRule::EvenOdd);
    let _ = cr.fill();
    cr.set_fill_rule(cairo::FillRule::Winding);
}

pub fn draw_line(cr: &cairo::Context, quad: &Quad) {
    let width = f64::from(quad.params[2].max(0.5));
    cr.new_path();
    cr.set_line_width(width);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.move_to(f64::from(quad.uv[0]), f64::from(quad.uv[1]));
    cr.line_to(f64::from(quad.uv[2]), f64::from(quad.uv[3]));
    set_rgba(cr, quad.color, 1.0);
    let _ = cr.stroke();
}

pub fn draw_shadow(cr: &cairo::Context, quad: &Quad) {
    let blur = quad.params[2].max(1.0);
    let radius = quad.params[1];
    for step in 1..=SHADOW_STEPS {
        let share = (SHADOW_STEPS + 1 - step) as f32 / SHADOW_STEPS as f32;
        let grow = blur * SHADOW_SPREAD * share;
        let ring = [
            quad.rect[0] - grow,
            quad.rect[1] - grow,
            quad.rect[2] + grow * 2.0,
            quad.rect[3] + grow * 2.0,
        ];
        cr.new_path();
        rounded_rect(cr, ring, radius + grow);
        set_rgba(cr, quad.color, SHADOW_STEP_ALPHA);
        let _ = cr.fill();
    }
}
