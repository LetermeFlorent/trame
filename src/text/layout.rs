use std::ops::Range;

use swash::shape::ShapeContext;
use swash::text::{Codepoint, Script};

use super::font::{Family, FontKey, FontLibrary, WEIGHT_AXIS};

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

struct Run {
    range: Range<usize>,
    font: FontKey,
    script: Option<Script>,
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

    fn shape_run(&mut self, text: &str, run: &Run, layout: &mut TextLayout) {
        let face = self.library.face(run.font.id);
        let mut builder = self
            .context
            .builder(face.font())
            .size(layout.style.size)
            .script(run.script.unwrap_or(Script::Latin));
        if let Some(weight) = run.font.weight {
            builder = builder.variations([(WEIGHT_AXIS, f32::from(weight))]);
        }
        let mut shaper = builder.build();
        shaper.add_str(text);
        let mut pen = layout.width;
        let glyphs = &mut layout.glyphs;
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                glyphs.push(LayoutGlyph {
                    font: run.font,
                    glyph: glyph.id,
                    x: pen + glyph.x,
                    y: glyph.y,
                });
                pen += glyph.advance;
            }
        });
        layout.width = pen;
    }
}

fn split_runs(
    library: &mut FontLibrary,
    primary: FontKey,
    text: &str,
    style: TextStyle,
) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for (start, ch) in text.char_indices() {
        let font = font_for(library, primary, ch, style);
        let script = Some(ch.script()).filter(|script| !is_neutral(*script));
        let end = start + ch.len_utf8();
        match runs.last_mut() {
            Some(run)
                if run.font == font
                    && (script.is_none() || run.script.is_none_or(|s| Some(s) == script)) =>
            {
                run.range.end = end;
                run.script = run.script.or(script);
            }
            _ => runs.push(Run {
                range: start..end,
                font,
                script,
            }),
        }
    }
    runs
}

fn font_for(library: &mut FontLibrary, primary: FontKey, ch: char, style: TextStyle) -> FontKey {
    if ch.is_control() {
        return primary;
    }
    library
        .covering_font(primary, ch, style.weight, false)
        .unwrap_or(primary)
}

fn is_neutral(script: Script) -> bool {
    matches!(script, Script::Common | Script::Inherited | Script::Unknown)
}
