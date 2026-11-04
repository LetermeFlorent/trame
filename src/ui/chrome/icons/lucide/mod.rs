mod chrome;
mod emblems;

pub(super) enum Element {
    Path(&'static str),
    Circle(f32, f32, f32),
    Rect(f32, f32, f32, f32, f32),
    Line(f32, f32, f32, f32),
    Polyline(&'static [f32]),
    Ellipse(f32, f32, f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Glyph {
    PanelLeftClose,
    PanelLeftOpen,
    Plus,
    X,
    Columns2,
    Minus,
    Square,
    Copy,
    SquareTerminal,
    Settings,
    ArrowDown,
    Pencil,
    Ban,
    LayoutGrid,
    Terminal,
    Folder,
    Code,
    Server,
    Globe,
    Database,
    Cog,
    ChevronDown,
    ChevronRight,
}

impl Glyph {
    pub(super) const ALL: [Glyph; 23] = [
        Glyph::PanelLeftClose,
        Glyph::PanelLeftOpen,
        Glyph::Plus,
        Glyph::X,
        Glyph::Columns2,
        Glyph::Minus,
        Glyph::Square,
        Glyph::Copy,
        Glyph::SquareTerminal,
        Glyph::Settings,
        Glyph::ArrowDown,
        Glyph::Pencil,
        Glyph::Ban,
        Glyph::LayoutGrid,
        Glyph::Terminal,
        Glyph::Folder,
        Glyph::Code,
        Glyph::Server,
        Glyph::Globe,
        Glyph::Database,
        Glyph::Cog,
        Glyph::ChevronDown,
        Glyph::ChevronRight,
    ];

    pub(super) fn source(self) -> &'static [Element] {
        chrome::source(self).unwrap_or_else(|| emblems::source(self))
    }
}
