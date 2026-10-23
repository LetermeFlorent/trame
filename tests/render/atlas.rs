use trame::render::{Atlas, AtlasKind};

const GLYPH: u32 = 200;

#[test]
fn a_cleared_atlas_starts_small_again() {
    let mut atlas = Atlas::new();
    let start = atlas.size(AtlasKind::Mask);
    while atlas.size(AtlasKind::Mask) == start {
        let pixels = vec![0; (GLYPH * GLYPH) as usize];
        let region = atlas.insert(AtlasKind::Mask, GLYPH, GLYPH, pixels);
        assert!(region.is_some());
    }
    atlas.clear();
    assert_eq!(atlas.size(AtlasKind::Mask), start);
    assert!(atlas.take_uploads().is_empty());
    assert!(!atlas.is_full());
}
