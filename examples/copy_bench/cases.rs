pub const SMALL_ROWS: usize = 40;
pub const SMALL_COLS: usize = 120;
pub const LARGE_ROWS: usize = 70;
pub const LARGE_COLS: usize = 280;
pub const MARK_ROWS: usize = 4;
pub const MARK_COLS: usize = 4096;

#[derive(Clone, Copy)]
pub enum Case {
    CopySmall,
    CopyLarge,
    CopyMarked,
    PaintMarked,
    PaintWide,
    XscriptMarked,
    XscriptWide,
}

pub const CASES: &[Case] = &[
    Case::CopySmall,
    Case::CopyLarge,
    Case::CopyMarked,
    Case::PaintMarked,
    Case::PaintWide,
    Case::XscriptMarked,
    Case::XscriptWide,
];

impl Case {
    pub fn name(self) -> String {
        match self {
            Self::CopySmall => format!("copy-{SMALL_ROWS}x{SMALL_COLS}"),
            Self::CopyLarge => format!("copy-{LARGE_ROWS}x{LARGE_COLS}"),
            Self::CopyMarked => format!("copy-{LARGE_ROWS}x{LARGE_COLS}-marks"),
            Self::PaintMarked => format!("paint-{LARGE_ROWS}x{LARGE_COLS}"),
            Self::PaintWide => format!("paint-{MARK_ROWS}x{MARK_COLS}"),
            Self::XscriptMarked => format!("xscript-{LARGE_ROWS}x{LARGE_COLS}"),
            Self::XscriptWide => format!("xscript-{MARK_ROWS}x{MARK_COLS}"),
        }
    }
}
