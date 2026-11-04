use crate::geometry::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(super) enum QuadKind {
    Fill = 0,
    Border = 1,
    Mask = 2,
    Color = 3,
    Line = 4,
    Shadow = 5,
}

#[derive(Clone, Copy)]
pub(super) struct Shape {
    pub(super) kind: QuadKind,
    pub(super) rect: Rect,
    pub(super) uv: [f32; 4],
    pub(super) radius: f32,
    pub(super) extent: f32,
}

impl Shape {
    pub(super) fn new(kind: QuadKind, rect: Rect) -> Self {
        Self {
            kind,
            rect,
            uv: [0.0; 4],
            radius: 0.0,
            extent: 0.0,
        }
    }

    pub(super) fn radius(self, radius: f32) -> Self {
        Self { radius, ..self }
    }

    pub(super) fn extent(self, extent: f32) -> Self {
        Self { extent, ..self }
    }
}
