use std::ops::Range;

use swash::shape::ShapeContext;
use swash::text::{Codepoint, Script};

use super::font::{Family, FontKey, FontLibrary, WEIGHT_AXIS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontRole {
    Terminal,
    Interface,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub role: FontRole,
    pub size: f32,
    pub weight: u16,
    pub italic: bool,
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

pub fn shape_text(
    context: &mut ShapeContext,
    library: &mut FontLibrary,
    family: &Family,
    text: &str,
    style: TextStyle,
) -> TextLayout {
    let mut layout = TextLayout {
        style,
        glyphs: Vec::new(),
        width: 0.0,
        ascent: 0.0,
        descent: 0.0,
        cap_height: 0.0,
    };
    let Some(primary) = family.resolve(library, style.weight, style.italic) else {
        return layout;
    };
    let metrics = library.face(primary.id).metrics(style.size);
    layout.ascent = metrics.ascent;
    layout.descent = metrics.descent;
    layout.cap_height = metrics.cap_height;
    for run in split_runs(library, primary, text, style) {
        shape_run(
            context,
            library,
            &text[run.range],
            run.font,
            run.script,
            &mut layout,
        );
    }
    layout
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
    if ch.is_control() || library.face(primary.id).covers(ch) {
        return primary;
    }
    match library.fallback_for(ch) {
        Some(id) => FontKey::synthesize(library, id, style.weight, style.italic),
        None => primary,
    }
}

fn shape_run(
    context: &mut ShapeContext,
    library: &FontLibrary,
    text: &str,
    font: FontKey,
    script: Option<Script>,
    layout: &mut TextLayout,
) {
    let face = library.face(font.id);
    let mut builder = context
        .builder(face.font())
        .size(layout.style.size)
        .script(script.unwrap_or(Script::Latin));
    if let Some(weight) = font.weight {
        builder = builder.variations([(WEIGHT_AXIS, f32::from(weight))]);
    }
    let mut shaper = builder.build();
    shaper.add_str(text);
    let mut pen = layout.width;
    let glyphs = &mut layout.glyphs;
    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            glyphs.push(LayoutGlyph {
                font,
                glyph: glyph.id,
                x: pen + glyph.x,
                y: glyph.y,
            });
            pen += glyph.advance;
        }
    });
    layout.width = pen;
}

fn is_neutral(script: Script) -> bool {
    matches!(script, Script::Common | Script::Inherited | Script::Unknown)
}
