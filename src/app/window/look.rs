use winit::window::{Theme, Window};

use trame::theme::{Appearance, system};

pub fn appearance(window: &Window) -> Appearance {
    window
        .theme()
        .map(appearance_of)
        .or_else(system::detect)
        .unwrap_or_default()
}

pub fn appearance_of(theme: Theme) -> Appearance {
    match theme {
        Theme::Light => Appearance::Light,
        Theme::Dark => Appearance::Dark,
    }
}
