mod picture;
mod preview;

use super::*;
use crate::text::{DEFAULT_LINE_HEIGHT, MAX_CACHED_GLYPHS};

#[test]
fn first_frame_overflow_does_not_permanently_cache_none() {
    let mut system = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("monospace font");
    system.atlas_mut().limit_to(8);
    let key1 = ImageKey {
        id: 1,
        width: 4,
        height: 4,
    };
    let key2 = ImageKey {
        id: 2,
        width: 4,
        height: 4,
    };
    assert!(system.image(key1, || vec![255; 64]).is_some());
    assert!(system.image(key2, || vec![255; 64]).is_none());
    assert!(!system.end_frame());
    system.atlas_mut().limit_to(64);
    assert!(system.image(key2, || vec![255; 64]).is_some());
}

#[test]
fn image_cache_bound_triggers_atlas_reset_and_clear() {
    let mut system = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("monospace font");
    assert!(!system.end_frame());
    for id in 0..=MAX_CACHED_GLYPHS as u64 {
        let key = ImageKey {
            id,
            width: 1,
            height: 1,
        };
        assert!(system.image(key, || vec![255; 4]).is_some());
    }
    assert!(system.end_frame());
    assert!(system.images.is_empty());
}

#[test]
fn color_image_cache_clears_after_recovery_invalidation() {
    let mut system = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("monospace font");
    let key = ImageKey {
        id: 9,
        width: 2,
        height: 2,
    };
    let mut generated = 0;
    let first = system.image(key, || {
        generated += 1;
        vec![255; 16]
    });
    assert!(first.is_some());
    assert!(!system.end_frame());
    let _ = system.atlas_mut().take_uploads();

    system.atlas_mut().limit_to(512);
    system.atlas_mut().invalidate();
    assert!(system.end_frame());
    let second = system.image(key, || {
        generated += 1;
        vec![128; 16]
    });
    assert!(second.is_some());
    assert_eq!(generated, 2);
    assert_eq!(system.atlas_mut().take_uploads().len(), 1);
}
