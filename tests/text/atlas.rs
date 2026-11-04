use trame::render::AtlasRegion;
use trame::text::TextSystem;

use crate::{cell, system};

const TINY_ATLAS: u32 = 32;

fn small_system() -> TextSystem {
    let mut text = system();
    text.atlas_mut().limit_to(TINY_ATLAS);
    text
}

fn draw_letters(text: &mut TextSystem) -> Vec<Option<AtlasRegion>> {
    ('a'..='z')
        .chain('A'..='Z')
        .map(|ch| text.cell_glyph(cell(ch, false, false, 1)))
        .map(|glyph| glyph.map(|glyph| glyph.region))
        .collect()
}

#[test]
fn a_full_atlas_keeps_the_glyphs_of_the_current_frame() {
    let mut text = small_system();
    let first = draw_letters(&mut text);
    let placed = first.iter().flatten().count();
    assert!(placed > 0 && placed < first.len(), "{placed} placed");
    assert!(!text.end_frame());
    assert_eq!(draw_letters(&mut text), first);
    assert_eq!(text.atlas_mut().take_uploads().len(), placed);
    assert!(!text.end_frame());
}

#[test]
fn a_full_atlas_drops_the_glyphs_of_older_frames() {
    let mut text = small_system();
    assert!(text.cell_glyph(cell('a', false, false, 1)).is_some());
    assert!(!text.end_frame());
    assert!(draw_letters(&mut text).contains(&None));
    assert!(text.end_frame());
    assert!(text.cell_glyph(cell('Z', false, false, 1)).is_some());
}
