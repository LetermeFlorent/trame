use std::collections::HashSet;

use trame::theme::Rgba;
use trame::theme::{Appearance, Swatch, Theme};

const MIN_CONTRAST: f32 = 3.0;
const MIN_TEXT_CONTRAST: f32 = 4.5;
const MIN_MUTED_CONTRAST: f32 = 4.0;

fn linear(channel: f32) -> f32 {
    match channel <= 0.04045 {
        true => channel / 12.92,
        false => ((channel + 0.055) / 1.055).powf(2.4),
    }
}

fn relative_luminance(color: Rgba) -> f32 {
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

fn contrast(first: Rgba, second: Rgba) -> f32 {
    let (a, b) = (relative_luminance(first), relative_luminance(second));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn themes() -> [Theme; 2] {
    [Theme::new(Appearance::Dark), Theme::new(Appearance::Light)]
}

fn over(tint: Rgba, backdrop: Rgba) -> Rgba {
    backdrop.mix(tint.with_alpha(1.0), tint.a)
}

#[test]
fn there_are_eight_distinct_swatches_in_each_theme() {
    assert_eq!(Swatch::ALL.len(), 8);
    for theme in themes() {
        let shades: HashSet<[u8; 3]> = Swatch::ALL
            .iter()
            .map(|swatch| theme.swatch(*swatch).to_rgb8())
            .collect();
        assert_eq!(shades.len(), 8);
    }
}

#[test]
fn every_shade_stands_out_on_the_chrome_and_the_terminal() {
    for theme in themes() {
        for swatch in Swatch::ALL {
            let shade = theme.swatch(swatch);
            for backdrop in [theme.chrome, theme.terminal, theme.tab_bar] {
                let ratio = contrast(shade, backdrop);
                assert!(
                    ratio >= MIN_CONTRAST,
                    "{swatch:?} on {:?}: {ratio}",
                    theme.appearance
                );
            }
        }
    }
}

#[test]
fn the_shades_differ_between_the_light_and_the_dark_theme() {
    let [dark, light] = themes();
    for swatch in Swatch::ALL {
        assert_ne!(dark.swatch(swatch), light.swatch(swatch));
    }
}

#[test]
fn a_tinted_row_keeps_its_name_readable() {
    for theme in themes() {
        for swatch in Swatch::ALL {
            let tinted = over(theme.swatch_tint(swatch), theme.chrome);
            for (text, minimum) in [
                (theme.text, MIN_TEXT_CONTRAST),
                (theme.text_muted, MIN_MUTED_CONTRAST),
            ] {
                let ratio = contrast(text, tinted);
                assert!(
                    ratio >= minimum,
                    "{swatch:?} on {:?}: {ratio}",
                    theme.appearance
                );
            }
        }
    }
}

#[test]
fn the_tint_is_the_swatch_colour_made_translucent() {
    for theme in themes() {
        for swatch in Swatch::ALL {
            let tint = theme.swatch_tint(swatch);
            assert_eq!(tint.with_alpha(1.0), theme.swatch(swatch));
            assert!(tint.a > 0.0 && tint.a < 0.3);
        }
    }
}

#[test]
fn swatches_serialize_as_lowercase_names() {
    let text = serde_json::to_string(&Swatch::Teal).expect("serializes");
    assert_eq!(text, "\"teal\"");
    let back: Swatch = serde_json::from_str(&text).expect("parses");
    assert_eq!(back, Swatch::Teal);
}
