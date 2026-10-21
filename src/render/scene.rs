use bytemuck::{Pod, Zeroable};

use super::atlas::{AtlasKind, AtlasRegion};
use crate::color::Rgba;
use crate::geometry::{Point, Rect};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Quad {
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub color: [f32; 4],
    pub params: [f32; 4],
    pub clip: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
enum QuadKind {
    Fill = 0,
    Border = 1,
    Mask = 2,
    Color = 3,
    Line = 4,
    Shadow = 5,
}

#[derive(Debug, Default)]
pub struct Scene {
    quads: Vec<Quad>,
    clips: Vec<Rect>,
    width: f32,
    height: f32,
}

impl Scene {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            ..Self::default()
        }
    }

    pub fn reset(&mut self, width: f32, height: f32) {
        self.quads.clear();
        self.clips.clear();
        self.width = width;
        self.height = height;
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn quads(&self) -> &[Quad] {
        &self.quads
    }

    pub fn clip(&self) -> Rect {
        self.clips
            .last()
            .copied()
            .unwrap_or(Rect::new(0.0, 0.0, self.width, self.height))
    }

    pub fn push_clip(&mut self, rect: Rect) {
        let clip = self.clip().intersect(rect);
        self.clips.push(clip);
    }

    pub fn pop_clip(&mut self) {
        self.clips.pop();
    }

    pub fn fill(&mut self, rect: Rect, color: Rgba, radius: f32) {
        self.push(Shape::new(QuadKind::Fill, rect).radius(radius), color);
    }

    pub fn border(&mut self, rect: Rect, color: Rgba, radius: f32, width: f32) {
        let shape = Shape::new(QuadKind::Border, rect)
            .radius(radius)
            .width(width);
        self.push(shape, color);
    }

    pub fn shadow(&mut self, rect: Rect, color: Rgba, radius: f32, blur: f32) {
        let shape = Shape::new(QuadKind::Shadow, rect)
            .radius(radius)
            .width(blur);
        self.push_within(shape, rect.inset(-blur * 2.0), color);
    }

    pub fn line(&mut self, from: Point, to: Point, width: f32, color: Rgba) {
        let reach = width / 2.0 + 1.0;
        let bounds = Rect::from_edges(
            from.x.min(to.x) - reach,
            from.y.min(to.y) - reach,
            from.x.max(to.x) + reach,
            from.y.max(to.y) + reach,
        );
        let mut shape = Shape::new(QuadKind::Line, bounds).width(width);
        shape.uv = [from.x, from.y, to.x, to.y];
        self.push(shape, color);
    }

    pub fn glyph(&mut self, region: AtlasRegion, x: f32, y: f32, color: Rgba) {
        let kind = match region.kind {
            AtlasKind::Mask => QuadKind::Mask,
            AtlasKind::Color => QuadKind::Color,
        };
        let rect = Rect::new(x, y, region.width as f32, region.height as f32);
        let mut shape = Shape::new(kind, rect);
        shape.uv = [region.x as f32, region.y as f32, 0.0, 0.0];
        self.push(shape, color);
    }

    fn push(&mut self, shape: Shape, color: Rgba) {
        self.push_within(shape, shape.rect, color);
    }

    fn push_within(&mut self, shape: Shape, reach: Rect, color: Rgba) {
        let clip = self.clip();
        if color.a <= 0.0 || shape.rect.is_empty() || reach.intersect(clip).is_empty() {
            return;
        }
        let rect = shape.rect;
        self.quads.push(Quad {
            rect: [rect.x, rect.y, rect.width, rect.height],
            uv: shape.uv,
            color: color.to_array(),
            params: [shape.kind as u32 as f32, shape.radius, shape.width, 0.0],
            clip: [clip.x, clip.y, clip.right(), clip.bottom()],
        });
    }
}

#[derive(Clone, Copy)]
struct Shape {
    kind: QuadKind,
    rect: Rect,
    uv: [f32; 4],
    radius: f32,
    width: f32,
}

impl Shape {
    fn new(kind: QuadKind, rect: Rect) -> Self {
        Self {
            kind,
            rect,
            uv: [0.0; 4],
            radius: 0.0,
            width: 0.0,
        }
    }

    fn radius(self, radius: f32) -> Self {
        Self { radius, ..self }
    }

    fn width(self, width: f32) -> Self {
        Self { width, ..self }
    }
}
