mod draw;
mod shape;

use bytemuck::{Pod, Zeroable};

use self::shape::Shape;
use crate::color::Rgba;
use crate::geometry::Rect;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Quad {
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub color: [f32; 4],
    pub params: [f32; 4],
    pub clip: [f32; 4],
}

#[derive(Debug, Default)]
pub struct Scene {
    quads: Vec<Quad>,
    clips: Vec<Rect>,
    opacities: Vec<f32>,
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
        self.opacities.clear();
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

    pub fn opacity(&self) -> f32 {
        self.opacities.last().copied().unwrap_or(1.0)
    }

    pub fn push_opacity(&mut self, opacity: f32) {
        let combined = self.opacity() * opacity.clamp(0.0, 1.0);
        self.opacities.push(combined);
    }

    pub fn pop_opacity(&mut self) {
        self.opacities.pop();
    }

    fn push(&mut self, shape: Shape, color: Rgba) {
        self.push_within(shape, shape.rect, color);
    }

    fn push_within(&mut self, shape: Shape, reach: Rect, color: Rgba) {
        let clip = self.clip();
        let color = color.with_alpha(color.a * self.opacity());
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
