use std::collections::HashSet;

use trame::theme::{Rgba, Swatch};

use crate::luminance::{MIN_CONTRAST, MIN_MUTED_CONTRAST, MIN_TEXT_CONTRAST, contrast, themes};

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

#[test]
fn a_custom_colour_is_stored_as_its_hex_code_and_a_preset_by_its_name() {
    let custom = Swatch::custom(0x12ab34);
    assert_eq!(String::from(custom), "#12AB34");
    assert_eq!(String::from(Swatch::Teal), "teal");
    assert_eq!(Swatch::parse("teal"), Some(Swatch::Teal));
    assert_eq!(Swatch::parse("#12ab34"), Some(custom));
    assert_eq!(Swatch::parse("mauve"), None);
}
