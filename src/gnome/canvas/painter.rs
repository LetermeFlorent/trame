use gtk::cairo;
use trame::render::{Atlas, AtlasKind, Quad};

use super::shapes::{
    clip_reach, draw_line, draw_shadow, fill_rounded, set_rgba, shadow_reach, stroke_border,
};
use super::uploads::{SurfaceSlot, ensure_slot, write_color, write_mask};

const KIND_FILL: u32 = 0;
const KIND_BORDER: u32 = 1;
const KIND_MASK: u32 = 2;
const KIND_COLOR: u32 = 3;
const KIND_LINE: u32 = 4;
const KIND_SHADOW: u32 = 5;

#[derive(Default)]
pub struct ScenePainter {
    mask: Option<SurfaceSlot>,
    color: Option<SurfaceSlot>,
}

impl ScenePainter {
    pub fn reset(&mut self) {
        self.mask = None;
        self.color = None;
    }

    pub fn sync_atlas(&mut self, atlas: &mut Atlas) {
        ensure_slot(&mut self.mask, cairo::Format::A8, atlas.size(AtlasKind::Mask));
        ensure_slot(&mut self.color, cairo::Format::ARgb32, atlas.size(AtlasKind::Color));
        for upload in atlas.take_uploads() {
            match upload.region.kind {
                AtlasKind::Mask => write_mask(self.mask.as_mut(), &upload),
                AtlasKind::Color => write_color(self.color.as_mut(), &upload),
            }
        }
    }

    pub fn paint(&self, cr: &cairo::Context, quads: &[Quad]) {
        for quad in quads {
            let kind = quad.params[0] as u32;
            let reach = if kind == KIND_SHADOW {
                shadow_reach(quad)
            } else {
                quad.rect
            };
            let _ = cr.save();
            if clip_reach(cr, quad.clip, reach) {
                self.paint_quad(cr, kind, quad);
            }
            let _ = cr.restore();
        }
    }

    fn paint_quad(&self, cr: &cairo::Context, kind: u32, quad: &Quad) {
        match kind {
            KIND_FILL => fill_rounded(cr, quad),
            KIND_BORDER => stroke_border(cr, quad),
            KIND_MASK => {
                if let Some(slot) = &self.mask {
                    paint_mask(cr, &slot.surface, quad);
                }
            }
            KIND_COLOR => {
                if let Some(slot) = &self.color {
                    paint_color(cr, &slot.surface, quad);
                }
            }
            KIND_LINE => draw_line(cr, quad),
            KIND_SHADOW => draw_shadow(cr, quad),
            _ => {}
        }
    }
}

fn paint_mask(cr: &cairo::Context, mask: &cairo::ImageSurface, quad: &Quad) {
    cr.rectangle(
        f64::from(quad.rect[0]),
        f64::from(quad.rect[1]),
        f64::from(quad.rect[2]),
        f64::from(quad.rect[3]),
    );
    cr.clip();
    set_rgba(cr, quad.color, 1.0);
    let ox = f64::from(quad.rect[0] - quad.uv[0]);
    let oy = f64::from(quad.rect[1] - quad.uv[1]);
    let _ = cr.mask_surface(mask, ox, oy);
}

fn paint_color(cr: &cairo::Context, color: &cairo::ImageSurface, quad: &Quad) {
    cr.rectangle(
        f64::from(quad.rect[0]),
        f64::from(quad.rect[1]),
        f64::from(quad.rect[2]),
        f64::from(quad.rect[3]),
    );
    cr.clip();
    let ox = f64::from(quad.rect[0] - quad.uv[0]);
    let oy = f64::from(quad.rect[1] - quad.uv[1]);
    let _ = cr.set_source_surface(color, ox, oy);
    let _ = cr.paint_with_alpha(f64::from(quad.color[3].clamp(0.0, 1.0)));
}
