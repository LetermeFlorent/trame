use trame::theme::Rgba;

use super::{ACCENT, TEXT};

pub type Sample = (&'static str, bool, bool, Rgba);

pub const LINES: &[Sample] = &[
    (
        "PS C:\\Projets\\trame> git status --short",
        false,
        false,
        TEXT,
    ),
    (
        " M src/text/system.rs   ?? examples/preview.rs",
        false,
        false,
        Rgba::hex(0xa1c181),
    ),
    (
        "Bold line: The quick brown fox jumps 0123456789",
        true,
        false,
        TEXT,
    ),
    (
        "Italic line: The quick brown fox jumps {}[]()",
        false,
        true,
        Rgba::hex(0xd07277),
    ),
    (
        "Bold italic: ->  =>  !=  <=  >=  ===  www",
        true,
        true,
        Rgba::hex(0xdec184),
    ),
    (
        "Unicode: café naïve Ελληνικά Кириллица → ✓ ✗ ★ ⚠",
        false,
        false,
        TEXT,
    ),
    ("Wide: 日本語 中文 한국어 😀 🚀 🎉", false, false, TEXT),
    (
        "Box: ┌──┬──┐ │ab│cd│ └──┴──┘ ▁▂▃▄▅▆▇█ ░▒▓",
        false,
        false,
        ACCENT,
    ),
];
