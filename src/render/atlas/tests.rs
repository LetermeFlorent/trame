use super::{Atlas, AtlasKind};

#[test]
fn invalidated_atlas_requests_reset_even_right_after_limit_to() {
    let mut atlas = Atlas::new();
    atlas.limit_to(1024);
    atlas.invalidate();
    assert!(atlas.finish_frame());
    atlas.clear();
    assert!(!atlas.finish_frame());
}

#[test]
fn invalidated_atlas_accepts_fresh_uploads_after_clear() {
    let mut atlas = Atlas::new();
    let first = atlas.insert(AtlasKind::Mask, 4, 4, vec![255; 16]);
    assert!(first.is_some());
    assert!(!atlas.finish_frame());
    atlas.limit_to(1024);
    atlas.invalidate();
    assert!(atlas.finish_frame());
    atlas.clear();
    let second = atlas.insert(AtlasKind::Mask, 4, 4, vec![128; 16]);
    assert!(second.is_some());
    assert_eq!(atlas.take_uploads().len(), 1);
    assert!(!atlas.finish_frame());
}

#[test]
fn limit_to_preserves_must_reset_after_invalidation() {
    let mut atlas = Atlas::new();
    atlas.invalidate();
    atlas.limit_to(1024);
    assert!(atlas.finish_frame());
    atlas.clear();
    assert!(!atlas.finish_frame());
}

#[test]
fn limit_to_requests_reset_when_atlas_holds_older_frames() {
    let mut atlas = Atlas::new();
    let region = atlas.insert(AtlasKind::Mask, 4, 4, vec![255; 16]);
    assert!(region.is_some());
    assert!(!atlas.finish_frame());
    atlas.limit_to(1024);
    assert!(atlas.finish_frame());
    atlas.clear();
    assert!(!atlas.finish_frame());
}
