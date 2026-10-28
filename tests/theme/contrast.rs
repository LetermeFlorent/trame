use trame::theme::{Appearance, Swatch};

use crate::swatches::{MIN_CONTRAST, MIN_TEXT_CONTRAST, contrast, themes};

#[test]
fn interface_text_reaches_4_5_to_1() {
    for theme in themes() {
        for text in [theme.text, theme.text_muted, theme.text_faint] {
            for surface in [theme.chrome, theme.tab_bar] {
                let found = contrast(text, surface);
                let appearance = theme.appearance;
                assert!(
                    found >= MIN_TEXT_CONTRAST,
                    "{appearance:?} {text:?}: {found}"
                );
            }
        }
    }
}

#[test]
fn labels_on_filled_buttons_reach_4_5_to_1() {
    for theme in themes() {
        for fill in [theme.accent, theme.swatch(Swatch::Orange)] {
            let found = contrast(fill.ink(), fill);
            let appearance = theme.appearance;
            assert!(
                found >= MIN_TEXT_CONTRAST,
                "{appearance:?} {fill:?}: {found}"
            );
        }
    }
}

#[test]
fn ansi_colors_but_the_dark_black_reach_3_to_1() {
    for theme in themes() {
        let printed = match theme.appearance {
            Appearance::Dark => &theme.ansi[1..],
            Appearance::Light => &theme.ansi[..],
        };
        for color in printed {
            let found = contrast(*color, theme.terminal);
            let appearance = theme.appearance;
            assert!(found >= MIN_CONTRAST, "{appearance:?} {color:?}: {found}");
        }
    }
}
