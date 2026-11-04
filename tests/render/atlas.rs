use trame::render::{Atlas, AtlasKind};

const GLYPH: u32 = 200;
const DEVICE_LIMIT: u32 = 1024;

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
    assert!(!atlas.finish_frame());
}

#[test]
fn an_atlas_stays_within_the_device_limit() {
    let mut atlas = Atlas::new();
    atlas.limit_to(DEVICE_LIMIT);
    let pixels = || vec![0; (GLYPH * GLYPH) as usize];
    while atlas
        .insert(AtlasKind::Mask, GLYPH, GLYPH, pixels())
        .is_some()
    {}
    assert_eq!(atlas.size(AtlasKind::Mask), DEVICE_LIMIT);
}
