use crate::text::{FontRole, REGULAR, SEMIBOLD, TextStyle};

pub const TERMINAL_FONT_SIZE: f32 = 14.0;
pub const MIN_TERMINAL_FONT_SIZE: f32 = 8.0;
pub const MAX_TERMINAL_FONT_SIZE: f32 = 32.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Mac,
    Linux,
}

impl Platform {
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Linux
        }
    }

    pub fn draws_window_controls(self) -> bool {
        self != Self::Mac
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMetrics {
    pub scale: f32,
    pub platform: Platform,
}

impl UiMetrics {
    pub const TITLE_BAR: f32 = 32.0;
    pub const SIDEBAR_WIDTH: f32 = 216.0;
    pub const SIDEBAR_HEADER: f32 = 30.0;
    pub const SIDEBAR_ROW: f32 = 26.0;
    pub const SIDEBAR_INDENT: f32 = 22.0;
    pub const TAB_BAR: f32 = 26.0;
    pub const TAB_PADDING: f32 = 10.0;
    pub const TAB_CLOSE: f32 = 16.0;
    pub const TAB_MIN: f32 = 64.0;
    pub const TAB_MAX: f32 = 200.0;
    pub const BUTTON: f32 = 22.0;
    pub const WINDOW_BUTTON: f32 = 46.0;
    pub const MAC_CONTROLS_INSET: f32 = 76.0;
    pub const TERMINAL_PAD_X: f32 = 10.0;
    pub const TERMINAL_PAD_Y: f32 = 6.0;
    pub const DIVIDER: f32 = 1.0;
    pub const DIVIDER_GRAB: f32 = 4.0;
    pub const RESIZE_BORDER: f32 = 5.0;
    pub const RADIUS: f32 = 5.0;
    pub const ICON: f32 = 14.0;
    pub const UI_FONT: f32 = 12.5;
    pub const SMALL_FONT: f32 = 11.0;

    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            platform: Platform::current(),
        }
    }

    pub fn px(&self, logical: f32) -> f32 {
        (logical * self.scale).round()
    }

    pub fn line(&self) -> f32 {
        self.scale.round().max(1.0)
    }

    pub fn label(&self) -> TextStyle {
        TextStyle {
            role: FontRole::Interface,
            size: Self::UI_FONT * self.scale,
            weight: REGULAR,
            italic: false,
        }
    }

    pub fn strong(&self) -> TextStyle {
        TextStyle {
            weight: SEMIBOLD,
            ..self.label()
        }
    }

    pub fn caption(&self) -> TextStyle {
        TextStyle {
            size: Self::SMALL_FONT * self.scale,
            weight: SEMIBOLD,
            ..self.label()
        }
    }
}
