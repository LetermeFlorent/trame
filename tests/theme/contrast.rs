use trame::theme::{Appearance, Swatch};

use crate::swatches::{MIN_CONTRAST, MIN_MUTED_CONTRAST, MIN_TEXT_CONTRAST, contrast, themes};

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

#[test]
fn labels_on_a_filled_tab_reach_4_to_1_on_every_swatch() {
    for theme in themes() {
        for swatch in Swatch::ALL {
            let fill = theme.swatch(swatch);
            let found = contrast(fill.ink(), fill);
            let appearance = theme.appearance;
            assert!(
                found >= MIN_MUTED_CONTRAST,
                "{appearance:?} {swatch:?}: {found}"
            );
        }
    }
}

#[test]
fn a_tinted_terminal_keeps_its_text_readable() {
    let extremes = [Swatch::custom(0xffffff), Swatch::custom(0x000000)];
    for theme in themes() {
        assert_eq!(theme.terminal_under(None), theme.terminal);
        for swatch in Swatch::ALL.into_iter().chain(extremes) {
            let tinted = theme.terminal_under(Some(swatch));
            assert_ne!(tinted, theme.terminal);
            let found = contrast(theme.foreground, tinted);
            let appearance = theme.appearance;
            assert!(
                found >= MIN_TEXT_CONTRAST,
                "{appearance:?} {swatch:?}: {found}"
            );
        }
    }
}

#[test]
fn tinting_changes_only_the_terminal_background() {
    for theme in themes() {
        let tinted = theme.tinted(Some(Swatch::Blue));
        assert_ne!(tinted.terminal, theme.terminal);
        assert_eq!(tinted.selection, theme.selection);
        assert_eq!(tinted.accent, theme.accent);
        assert_eq!(tinted.foreground, theme.foreground);
        assert_eq!(tinted.ansi, theme.ansi);
        assert_eq!(theme.tinted(None), theme);
    }
}
