use super::Element::{self, Circle, Path, Rect};
use super::Glyph;

const FRAME: Element = Rect(3.0, 3.0, 18.0, 18.0, 2.0);

const PANEL_LEFT_CLOSE: &[Element] = &[FRAME, Path("M9 3v18"), Path("m16 15-3-3 3-3")];
const PANEL_LEFT_OPEN: &[Element] = &[FRAME, Path("M9 3v18"), Path("m14 9 3 3-3 3")];
const PLUS: &[Element] = &[Path("M5 12h14"), Path("M12 5v14")];
const X: &[Element] = &[Path("M18 6 6 18"), Path("m6 6 12 12")];
const COLUMNS_2: &[Element] = &[FRAME, Path("M12 3v18")];
const MINUS: &[Element] = &[Path("M5 12h14")];
const SQUARE: &[Element] = &[FRAME];
const COPY: &[Element] = &[
    Rect(8.0, 8.0, 14.0, 14.0, 2.0),
    Path("M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"),
];
const SQUARE_TERMINAL: &[Element] = &[Path("m7 11 2-2-2-2"), Path("M11 13h4"), FRAME];
const SETTINGS: &[Element] = &[
    Path(
        "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
    ),
    Circle(12.0, 12.0, 3.0),
];
const ARROW_DOWN: &[Element] = &[Path("M12 5v14"), Path("m19 12-7 7-7-7")];
const PENCIL: &[Element] = &[
    Path(
        "M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z",
    ),
    Path("m15 5 4 4"),
];
const BAN: &[Element] = &[Circle(12.0, 12.0, 10.0), Path("m4.9 4.9 14.2 14.2")];

pub(super) fn source(glyph: Glyph) -> Option<&'static [Element]> {
    Some(match glyph {
        Glyph::PanelLeftClose => PANEL_LEFT_CLOSE,
        Glyph::PanelLeftOpen => PANEL_LEFT_OPEN,
        Glyph::Plus => PLUS,
        Glyph::X => X,
        Glyph::Columns2 => COLUMNS_2,
        Glyph::Minus => MINUS,
        Glyph::Square => SQUARE,
        Glyph::Copy => COPY,
        Glyph::SquareTerminal => SQUARE_TERMINAL,
        Glyph::Settings => SETTINGS,
        Glyph::ArrowDown => ARROW_DOWN,
        Glyph::Pencil => PENCIL,
        Glyph::Ban => BAN,
        _ => return None,
    })
}
