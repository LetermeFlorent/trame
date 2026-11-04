use trame::theme::{Appearance, Rgba, Theme};

pub(crate) const MIN_CONTRAST: f32 = 3.0;
pub(crate) const MIN_TEXT_CONTRAST: f32 = 4.5;
pub(crate) const MIN_MUTED_CONTRAST: f32 = 4.0;

fn linear(channel: f32) -> f32 {
    match channel <= 0.04045 {
        true => channel / 12.92,
        false => ((channel + 0.055) / 1.055).powf(2.4),
    }
}

fn relative_luminance(color: Rgba) -> f32 {
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

pub(crate) fn contrast(first: Rgba, second: Rgba) -> f32 {
    let (a, b) = (relative_luminance(first), relative_luminance(second));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

pub(crate) fn themes() -> [Theme; 2] {
    [Theme::new(Appearance::Dark), Theme::new(Appearance::Light)]
}
