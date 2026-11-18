use super::Scene;
use super::shape::{QuadKind, Shape};
use crate::geometry::Rect;
use crate::term::graphics::Image;
use crate::theme::Rgba;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageDraw {
    pub quad: u32,
    pub serial: u64,
}

impl Scene {
    pub fn image(&mut self, image: &Image, dst: Rect, uv: [f32; 4]) {
        let quad = self.quads.len() as u32;
        let mut shape = Shape::new(QuadKind::Image, dst);
        shape.uv = uv;
        if !self.push(shape, Rgba::WHITE) {
            return;
        }
        let serial = image.serial;
        self.images.push(ImageDraw { quad, serial });
        if !self.uploads.iter().any(|upload| upload.serial == serial) {
            self.uploads.push(image.clone());
        }
    }

    pub fn images(&self) -> &[ImageDraw] {
        &self.images
    }

    pub fn image_uploads(&self) -> &[Image] {
        &self.uploads
    }
}
