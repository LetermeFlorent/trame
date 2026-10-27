use super::{Appearance, Theme};
use crate::theme::Rgba;

pub fn dark() -> Theme {
    let accent = Rgba::hex(0x74ade8);
    Theme {
        appearance: Appearance::Dark,
        chrome: Rgba::hex(0x1b1e23),
        tab_bar: Rgba::hex(0x1f2228),
        terminal: Rgba::hex(0x24282f),
        foreground: Rgba::hex(0xd5d9e0),
        border: Rgba::hex(0x2d3139),
        border_strong: Rgba::hex(0x3a3f49),
        text: Rgba::hex(0xdce0e5),
        text_muted: Rgba::hex(0x9ca3af),
        text_faint: Rgba::hex(0x6b7280),
        accent,
        hover: Rgba::hex(0x2a2e36),
        pressed: Rgba::hex(0x323741),
        selection: accent.with_alpha(0.3),
        drop_target: accent.with_alpha(0.16),
        shadow: Rgba::new(0.0, 0.0, 0.0, 0.45),
        ansi: DARK_ANSI,
    }
}

pub fn light() -> Theme {
    let accent = Rgba::hex(0x3b7ddd);
    Theme {
        appearance: Appearance::Light,
        chrome: Rgba::hex(0xebecef),
        tab_bar: Rgba::hex(0xf1f2f4),
        terminal: Rgba::hex(0xfbfbfc),
        foreground: Rgba::hex(0x2c313a),
        border: Rgba::hex(0xdcdfe4),
        border_strong: Rgba::hex(0xc8ccd3),
        text: Rgba::hex(0x24292f),
        text_muted: Rgba::hex(0x5c6370),
        text_faint: Rgba::hex(0x8b919c),
        accent,
        hover: Rgba::hex(0xe1e3e8),
        pressed: Rgba::hex(0xd6d9df),
        selection: accent.with_alpha(0.22),
        drop_target: accent.with_alpha(0.12),
        shadow: Rgba::new(0.05, 0.07, 0.1, 0.18),
        ansi: LIGHT_ANSI,
    }
}

const DARK_ANSI: [Rgba; 16] = [
    Rgba::hex(0x3b414d),
    Rgba::hex(0xd07277),
    Rgba::hex(0xa1c181),
    Rgba::hex(0xdec184),
    Rgba::hex(0x74ade8),
    Rgba::hex(0xb477cf),
    Rgba::hex(0x6eb4bf),
    Rgba::hex(0xc5cad3),
    Rgba::hex(0x6b7280),
    Rgba::hex(0xe8898e),
    Rgba::hex(0xb9d69a),
    Rgba::hex(0xecd49c),
    Rgba::hex(0x93c2f0),
    Rgba::hex(0xc994de),
    Rgba::hex(0x88c8d2),
    Rgba::hex(0xf3f4f6),
];

const LIGHT_ANSI: [Rgba; 16] = [
    Rgba::hex(0x2c313a),
    Rgba::hex(0xc4453f),
    Rgba::hex(0x3d8a3a),
    Rgba::hex(0x9c6a00),
    Rgba::hex(0x2f6fcf),
    Rgba::hex(0x9a42b8),
    Rgba::hex(0x1d8494),
    Rgba::hex(0x8f959f),
    Rgba::hex(0x6b7280),
    Rgba::hex(0xd65b55),
    Rgba::hex(0x4b9d47),
    Rgba::hex(0xb27c05),
    Rgba::hex(0x3b7ddd),
    Rgba::hex(0xad57cb),
    Rgba::hex(0x2896a6),
    Rgba::hex(0xb5bac3),
];
