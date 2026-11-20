use super::draw;
use trame::render::{AtlasKind, Gpu, OFFSCREEN_FORMAT, Renderer};
use trame::text::{DEFAULT_LINE_HEIGHT, TextSystem};

#[test]
fn ghost_cursor_retries_on_atlas_overflow_holding_older_frames() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let gpu = Gpu::headless().expect("gpu");
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    let expected = draw(false, &gpu, &mut renderer, &mut text);

    text.forget_glyphs();
    text.atlas_mut().limit_to(64);
    let fill = vec![255; 63 * 63];
    let slot = text.atlas_mut().insert(AtlasKind::Mask, 63, 63, fill);
    assert!(slot.is_some());
    renderer.sync_atlas(&gpu, text.atlas_mut());
    assert!(!text.end_frame());

    let actual = draw(false, &gpu, &mut renderer, &mut text);
    assert!(expected == actual, "ghost cursor dropped glyphs");
    assert!(!text.end_frame());
}
