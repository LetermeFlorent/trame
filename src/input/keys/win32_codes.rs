pub(super) mod virtual_key {
    pub const BACK: u16 = 0x08;
    pub const TAB: u16 = 0x09;
    pub const RETURN: u16 = 0x0d;
    pub const ESCAPE: u16 = 0x1b;
    pub const SPACE: u16 = 0x20;
    pub const PRIOR: u16 = 0x21;
    pub const NEXT: u16 = 0x22;
    pub const END: u16 = 0x23;
    pub const HOME: u16 = 0x24;
    pub const LEFT: u16 = 0x25;
    pub const UP: u16 = 0x26;
    pub const RIGHT: u16 = 0x27;
    pub const DOWN: u16 = 0x28;
    pub const INSERT: u16 = 0x2d;
    pub const DELETE: u16 = 0x2e;
    pub const F1: u16 = 0x70;
    pub const F11: u16 = 0x7a;
    pub const F12: u16 = 0x7b;
}

pub(super) mod scan_code {
    pub const ESCAPE: u16 = 0x01;
    pub const BACKSPACE: u16 = 0x0e;
    pub const TAB: u16 = 0x0f;
    pub const ENTER: u16 = 0x1c;
    pub const SPACE: u16 = 0x39;
    pub const F1: u16 = 0x3b;
    pub const HOME: u16 = 0x47;
    pub const UP: u16 = 0x48;
    pub const PAGE_UP: u16 = 0x49;
    pub const LEFT: u16 = 0x4b;
    pub const RIGHT: u16 = 0x4d;
    pub const END: u16 = 0x4f;
    pub const DOWN: u16 = 0x50;
    pub const PAGE_DOWN: u16 = 0x51;
    pub const INSERT: u16 = 0x52;
    pub const DELETE: u16 = 0x53;
    pub const F11: u16 = 0x57;
    pub const F12: u16 = 0x58;
}

pub(super) mod unicode {
    pub const NONE: u16 = 0;
    pub const BACKSPACE: u16 = 0x08;
    pub const TAB: u16 = 0x09;
    pub const LINE_FEED: u16 = 0x0a;
    pub const CARRIAGE_RETURN: u16 = 0x0d;
    pub const ESCAPE: u16 = 0x1b;
    pub const SPACE: u16 = 0x20;
    pub const DELETE: u16 = 0x7f;
}
