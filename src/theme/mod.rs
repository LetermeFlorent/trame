mod ansi;
mod palettes;
pub mod system;

use crate::color::Rgba;
use crate::term::{Color, ReportedColors};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Appearance {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub appearance: Appearance,
    pub chrome: Rgba,
    pub tab_bar: Rgba,
    pub tab_active: Rgba,
    pub terminal: Rgba,
    pub foreground: Rgba,
    pub border: Rgba,
    pub border_strong: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub text_faint: Rgba,
    pub accent: Rgba,
    pub hover: Rgba,
    pub pressed: Rgba,
    pub selection: Rgba,
    pub cursor: Rgba,
    pub drop_target: Rgba,
    pub shadow: Rgba,
    pub danger: Rgba,
    pub ansi: [Rgba; 16],
}

impl Theme {
    pub fn new(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Dark => palettes::dark(),
            Appearance::Light => palettes::light(),
        }
    }

    pub fn terminal_color(&self, color: Color) -> Rgba {
        match color {
            Color::Foreground => self.foreground,
            Color::Background => self.terminal,
            Color::Indexed(index) => self.indexed(index),
            Color::Rgb(red, green, blue) => Rgba::rgb8(red, green, blue),
        }
    }

    pub fn indexed(&self, index: u8) -> Rgba {
        self.ansi
            .get(usize::from(index))
            .copied()
            .unwrap_or_else(|| ansi::extended(index))
    }

    pub fn reported_colors(&self) -> ReportedColors {
        ReportedColors {
            foreground: self.foreground.to_rgb8(),
            background: self.terminal.to_rgb8(),
            cursor: self.cursor.to_rgb8(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
