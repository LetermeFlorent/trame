pub(super) const MODE_UNKNOWN: u8 = 0;
pub(super) const MODE_SET: u8 = 1;
pub(super) const MODE_RESET: u8 = 2;

pub(super) mod ansi_mode {
    pub const INSERT: u16 = 4;
    pub const NEW_LINE: u16 = 20;
}

pub(super) mod private_mode {
    pub const APP_CURSOR: u16 = 1;
    pub const ORIGIN: u16 = 6;
    pub const AUTOWRAP: u16 = 7;
    pub const CURSOR_VISIBLE: u16 = 25;
    pub const ALT_SCREEN_LEGACY: u16 = 47;
    pub const MOUSE_CLICK: u16 = 1000;
    pub const MOUSE_DRAG: u16 = 1002;
    pub const MOUSE_MOTION: u16 = 1003;
    pub const FOCUS_EVENTS: u16 = 1004;
    pub const MOUSE_SGR: u16 = 1006;
    pub const ALTERNATE_SCROLL: u16 = 1007;
    pub const ALT_SCREEN: u16 = 1047;
    pub const SAVE_CURSOR: u16 = 1048;
    pub const ALT_SCREEN_AND_CURSOR: u16 = 1049;
    pub const BRACKETED_PASTE: u16 = 2004;
    pub const SYNC_UPDATE: u16 = 2026;
    pub const WIN32_INPUT: u16 = 9001;
}

pub(super) mod sgr {
    pub const RESET: u16 = 0;
    pub const BOLD: u16 = 1;
    pub const DIM: u16 = 2;
    pub const ITALIC: u16 = 3;
    pub const UNDERLINE: u16 = 4;
    pub const INVERSE: u16 = 7;
    pub const HIDDEN: u16 = 8;
    pub const STRIKE: u16 = 9;
    pub const DOUBLE_UNDERLINE: u16 = 21;
    pub const NORMAL_INTENSITY: u16 = 22;
    pub const NO_ITALIC: u16 = 23;
    pub const NO_UNDERLINE: u16 = 24;
    pub const NO_INVERSE: u16 = 27;
    pub const NO_HIDDEN: u16 = 28;
    pub const NO_STRIKE: u16 = 29;
    pub const FOREGROUND_FIRST: u16 = 30;
    pub const FOREGROUND_LAST: u16 = 37;
    pub const FOREGROUND_EXTENDED: u16 = 38;
    pub const FOREGROUND_DEFAULT: u16 = 39;
    pub const BACKGROUND_FIRST: u16 = 40;
    pub const BACKGROUND_LAST: u16 = 47;
    pub const BACKGROUND_EXTENDED: u16 = 48;
    pub const BACKGROUND_DEFAULT: u16 = 49;
    pub const UNDERLINE_COLOR: u16 = 58;
    pub const BRIGHT_FOREGROUND_FIRST: u16 = 90;
    pub const BRIGHT_FOREGROUND_LAST: u16 = 97;
    pub const BRIGHT_BACKGROUND_FIRST: u16 = 100;
    pub const BRIGHT_BACKGROUND_LAST: u16 = 107;
    pub const BRIGHT_OFFSET: u16 = 8;
    pub const UNDERLINE_NONE: u16 = 0;
    pub const UNDERLINE_SINGLE: u16 = 1;
    pub const UNDERLINE_DOUBLE: u16 = 2;
    pub const UNDERLINE_CURLY: u16 = 3;
    pub const COLOR_INDEXED: u16 = 5;
    pub const COLOR_RGB: u16 = 2;
    pub const RGB_PARTS: usize = 3;
    pub const COLON_RGB_LEN: usize = 4;
    pub const COLON_RGB_WITH_SPACE_LEN: usize = 5;
}
