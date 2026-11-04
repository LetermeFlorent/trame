mod lucide;
mod outline;
mod painting;
mod path;
mod stroke;
#[cfg(test)]
mod tests;

use crate::geometry::Rect;
use crate::render::Scene;
use crate::theme::{Emblem, Rgba};
use lucide::Glyph;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Sidebar { open: bool },
    Plus,
    Close,
    SplitRight,
    Minimize,
    Maximize,
    Restore,
    Terminal,
    Gear,
    Update,
    Pencil,
    Ban,
    Emblem(Emblem),
}

pub struct Pen<'a> {
    pub scene: &'a mut Scene,
    pub color: Rgba,
    pub stroke: f32,
}

fn glyph(icon: Icon) -> Glyph {
    match icon {
        Icon::Sidebar { open: true } => Glyph::PanelLeftClose,
        Icon::Sidebar { open: false } => Glyph::PanelLeftOpen,
        Icon::Plus => Glyph::Plus,
        Icon::Close => Glyph::X,
        Icon::SplitRight => Glyph::Columns2,
        Icon::Minimize => Glyph::Minus,
        Icon::Maximize => Glyph::Square,
        Icon::Restore => Glyph::Copy,
        Icon::Terminal => Glyph::SquareTerminal,
        Icon::Gear => Glyph::Settings,
        Icon::Update => Glyph::ArrowDown,
        Icon::Pencil => Glyph::Pencil,
        Icon::Ban => Glyph::Ban,
        Icon::Emblem(emblem) => match emblem {
            Emblem::Grid | Emblem::Letters(_) | Emblem::Image(_) => Glyph::LayoutGrid,
            Emblem::Terminal => Glyph::Terminal,
            Emblem::Folder => Glyph::Folder,
            Emblem::Code => Glyph::Code,
            Emblem::Server => Glyph::Server,
            Emblem::Globe => Glyph::Globe,
            Emblem::Database => Glyph::Database,
            Emblem::Gear => Glyph::Cog,
        },
    }
}

pub fn draw_icon(pen: &mut Pen, icon: Icon, rect: Rect) {
    stroke::draw(pen, outline::outline(glyph(icon)), rect);
}
