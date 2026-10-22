use super::Scene;
use super::shape::{QuadKind, Shape};
use crate::geometry::{Point, Rect};
use crate::render::atlas::{AtlasKind, AtlasRegion};
use crate::theme::Rgba;

impl Scene {
    pub fn fill(&mut self, rect: Rect, color: Rgba, radius: f32) {
        self.push(Shape::new(QuadKind::Fill, rect).radius(radius), color);
    }

    pub fn fill_top(&mut self, rect: Rect, color: Rgba, radius: f32) {
        self.push_clip(rect);
        let reach = Rect::new(rect.x, rect.y, rect.width, rect.height + radius);
        self.fill(reach, color, radius);
        self.pop_clip();
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
}
