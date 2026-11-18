use crate::theme::Rgba;

const SRGB_LINEAR_LIMIT: f64 = 0.04045;
const SRGB_LINEAR_SLOPE: f64 = 12.92;
const SRGB_OFFSET: f64 = 0.055;
const SRGB_GAMMA: f64 = 2.4;

pub(super) fn clear_color(color: Rgba, srgb_output: bool) -> wgpu::Color {
    let channel = |value: f32| {
        let value = f64::from(value);
        if srgb_output {
            srgb_to_linear(value)
        } else {
            value
        }
    };
    wgpu::Color {
        r: channel(color.r),
        g: channel(color.g),
        b: channel(color.b),
        a: f64::from(color.a),
    }
}

fn srgb_to_linear(value: f64) -> f64 {
    if value <= SRGB_LINEAR_LIMIT {
        value / SRGB_LINEAR_SLOPE
    } else {
        ((value + SRGB_OFFSET) / (1.0 + SRGB_OFFSET)).powf(SRGB_GAMMA)
    }
}
