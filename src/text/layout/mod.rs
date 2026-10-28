mod runs;

use swash::shape::ShapeContext;
use swash::shape::cluster::Glyph;
use swash::text::Script;

use super::font::{Family, FontKey, FontLibrary, WEIGHT_AXIS};
use runs::{Run, split_runs};

pub use runs::script_of;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontRole {
    Interface,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub role: FontRole,
    pub size: f32,
    pub weight: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct LayoutGlyph {
    pub font: FontKey,
    pub glyph: u16,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct TextLayout {
    pub style: TextStyle,
    pub glyphs: Vec<LayoutGlyph>,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub cap_height: f32,
}

pub struct Shaping<'a> {
    pub context: &'a mut ShapeContext,
    pub library: &'a mut FontLibrary,
}

impl Shaping<'_> {
    pub fn shape(mut self, family: &Family, text: &str, style: TextStyle) -> TextLayout {
        let mut layout = TextLayout {
            style,
            glyphs: Vec::new(),
            width: 0.0,
            ascent: 0.0,
            descent: 0.0,
            cap_height: 0.0,
        };
        let Some(primary) = family.resolve(self.library, style.weight, false) else {
            return layout;
        };
        let metrics = self.library.face(primary.id).metrics(style.size);
        layout.ascent = metrics.ascent;
        layout.descent = metrics.descent;
        layout.cap_height = metrics.cap_height;
        for run in split_runs(self.library, primary, text, style) {
            self.shape_run(&text[run.range.clone()], &run, &mut layout);
        }
        layout
    }

    pub fn glyphs(
        &mut self,
        font: FontKey,
        size: f32,
        script: Option<Script>,
        text: &str,
    ) -> Vec<Glyph> {
        let face = self.library.face(font.id);
        let mut builder = self
            .context
            .builder(face.font())
            .size(size)
            .script(script.unwrap_or(Script::Latin));
        if let Some(weight) = font.weight {
            builder = builder.variations([(WEIGHT_AXIS, f32::from(weight))]);
        }
        let mut shaper = builder.build();
        shaper.add_str(text);
        let mut glyphs = Vec::new();
        shaper.shape_with(|cluster| glyphs.extend_from_slice(cluster.glyphs));
        glyphs
    }

    fn shape_run(&mut self, text: &str, run: &Run, layout: &mut TextLayout) {
        let mut pen = layout.width;
        for glyph in self.glyphs(run.font, layout.style.size, run.script, text) {
            layout.glyphs.push(LayoutGlyph {
                font: run.font,
                glyph: glyph.id,
                x: pen + glyph.x,
                y: glyph.y,
            });
            pen += glyph.advance;
        }
        layout.width = pen;
    }
}
