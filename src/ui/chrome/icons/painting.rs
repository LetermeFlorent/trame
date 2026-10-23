use super::{Icon, Pen, draw_icon};
use crate::files::{ICON_SIZE, fit};
use crate::geometry::{Point, Rect};
use crate::text::{ImageKey, TextStyle};
use crate::theme::{Emblem, Letters, Rgba};
use crate::ui::chrome::painter::Painter;

const ICON_STROKE: f32 = 1.25;
const ONE_LETTER: f32 = 0.95;
const TWO_LETTERS: f32 = 0.78;
const IMAGE_TINT: Rgba = Rgba::hex(0xffffff);

impl Painter<'_> {
    pub(in crate::ui::chrome) fn icon(&mut self, icon: Icon, rect: Rect, color: Rgba) {
        match icon {
            Icon::Emblem(Emblem::Letters(letters)) => self.letters(letters, rect, color),
            Icon::Emblem(Emblem::Image(id)) if self.image(id, rect) => {}
            _ => self.stroke(icon, rect, color),
        }
    }

    fn stroke(&mut self, icon: Icon, rect: Rect, color: Rgba) {
        let stroke = (ICON_STROKE * self.metrics.scale).round().max(1.0);
        let mut pen = Pen {
            scene: self.scene,
            color,
            stroke,
        };
        draw_icon(&mut pen, icon, rect);
    }

    fn letters(&mut self, letters: Letters, rect: Rect, color: Rgba) {
        let share = if letters.count() == 1 {
            ONE_LETTER
        } else {
            TWO_LETTERS
        };
        let text = letters.to_string();
        let mut style = TextStyle {
            size: rect.height * share,
            ..self.metrics.strong()
        };
        let mut layout = self.text.layout(&text, style);
        if layout.width > rect.width {
            style.size *= rect.width / layout.width;
            layout = self.text.layout(&text, style);
        }
        let x = rect.x + (rect.width - layout.width) / 2.0;
        let y = rect.y + (rect.height + layout.cap_height) / 2.0;
        let pen = Point::new(x.round(), y.round());
        self.text.draw_layout(self.scene, &layout, pen, color);
    }

    fn image(&mut self, id: u64, rect: Rect) -> bool {
        let icons = self.icons;
        let Some(pixels) = icons.pixels(id) else {
            return false;
        };
        let size = rect.width.min(rect.height).round().max(1.0) as u32;
        let key = ImageKey { id, size };
        let Some(region) = self
            .text
            .image(key, || fit(pixels, ICON_SIZE, ICON_SIZE, size))
        else {
            return false;
        };
        let x = rect.x + (rect.width - size as f32) / 2.0;
        let y = rect.y + (rect.height - size as f32) / 2.0;
        self.scene.glyph(region, x.round(), y.round(), IMAGE_TINT);
        true
    }
}
