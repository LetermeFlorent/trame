use super::Element::{self, Circle, Ellipse, Line, Path, Polyline, Rect};
use super::Glyph;

const LAYOUT_GRID: &[Element] = &[
    Rect(3.0, 3.0, 7.0, 7.0, 1.0),
    Rect(14.0, 3.0, 7.0, 7.0, 1.0),
    Rect(14.0, 14.0, 7.0, 7.0, 1.0),
    Rect(3.0, 14.0, 7.0, 7.0, 1.0),
];
const TERMINAL: &[Element] = &[
    Polyline(&[4.0, 17.0, 10.0, 11.0, 4.0, 5.0]),
    Line(12.0, 19.0, 20.0, 19.0),
];
const FOLDER: &[Element] = &[Path(
    "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z",
)];
const CODE: &[Element] = &[
    Polyline(&[16.0, 18.0, 22.0, 12.0, 16.0, 6.0]),
    Polyline(&[8.0, 6.0, 2.0, 12.0, 8.0, 18.0]),
];
const SERVER: &[Element] = &[
    Rect(2.0, 2.0, 20.0, 8.0, 2.0),
    Rect(2.0, 14.0, 20.0, 8.0, 2.0),
    Line(6.0, 6.0, 6.01, 6.0),
    Line(6.0, 18.0, 6.01, 18.0),
];
const GLOBE: &[Element] = &[
    Circle(12.0, 12.0, 10.0),
    Path("M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"),
    Path("M2 12h20"),
];
const DATABASE: &[Element] = &[
    Ellipse(12.0, 5.0, 9.0, 3.0),
    Path("M3 5V19A9 3 0 0 0 21 19V5"),
    Path("M3 12A9 3 0 0 0 21 12"),
];
const COG: &[Element] = &[
    Path("M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z"),
    Path("M12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"),
    Path("M12 2v2"),
    Path("M12 22v-2"),
    Path("m17 20.66-1-1.73"),
    Path("M11 10.27 7 3.34"),
    Path("m20.66 17-1.73-1"),
    Path("m3.34 7 1.73 1"),
    Path("M14 12h8"),
    Path("M2 12h2"),
    Path("m20.66 7-1.73 1"),
    Path("m3.34 17 1.73-1"),
    Path("m17 3.34-1 1.73"),
    Path("m11 13.73-4 6.93"),
];

pub(super) fn source(glyph: Glyph) -> &'static [Element] {
    match glyph {
        Glyph::LayoutGrid => LAYOUT_GRID,
        Glyph::Terminal => TERMINAL,
        Glyph::Folder => FOLDER,
        Glyph::Code => CODE,
        Glyph::Server => SERVER,
        Glyph::Globe => GLOBE,
        Glyph::Database => DATABASE,
        Glyph::Cog => COG,
        _ => &[],
    }
}
