use trame::term::Color;
use trame::theme::{Appearance, Theme};

#[test]
fn the_first_sixteen_colors_come_from_the_theme() {
    let theme = Theme::new(Appearance::Dark);
    assert_eq!(theme.indexed(1), theme.ansi[1]);
    assert_eq!(theme.indexed(15), theme.ansi[15]);
}

#[test]
fn the_color_cube_and_gray_ramp_are_standard() {
    let theme = Theme::new(Appearance::Light);
    assert_eq!(theme.indexed(16).to_rgb8(), [0, 0, 0]);
    assert_eq!(theme.indexed(196).to_rgb8(), [255, 0, 0]);
    assert_eq!(theme.indexed(231).to_rgb8(), [255, 255, 255]);
    assert_eq!(theme.indexed(232).to_rgb8(), [8, 8, 8]);
    assert_eq!(theme.indexed(255).to_rgb8(), [238, 238, 238]);
}

#[test]
fn default_colors_follow_the_theme() {
    let theme = Theme::new(Appearance::Dark);
    assert_eq!(theme.terminal_color(Color::Foreground), theme.foreground);
    assert_eq!(theme.terminal_color(Color::Background), theme.terminal);
    assert_eq!(
        theme.terminal_color(Color::Rgb(1, 2, 3)).to_rgb8(),
        [1, 2, 3]
    );
}

#[test]
fn light_and_dark_themes_differ_in_brightness() {
    let dark = Theme::new(Appearance::Dark);
    let light = Theme::new(Appearance::Light);
    assert!(dark.terminal.luminance() < 0.2);
    assert!(light.terminal.luminance() > 0.8);
    assert!(dark.text.luminance() > dark.terminal.luminance());
    assert!(light.text.luminance() < light.terminal.luminance());
}
