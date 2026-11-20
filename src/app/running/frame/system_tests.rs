use std::sync::Arc;

use trame::render::{Gpu, OFFSCREEN_FORMAT, Renderer, render_in_format};
use trame::text::{DEFAULT_LINE_HEIGHT, ImageKey, TextSystem};
use trame::theme::{Appearance, Rgba, Theme};
use trame::ui::UiMetrics;
use trame::ui::drop::ghost_scene;

use super::Running;
use crate::app::running::pointer::selection::PictureView;

const TEST_KEY: ImageKey = ImageKey {
    id: 7,
    width: 4,
    height: 4,
};

fn render_ghost(
    gpu: &Gpu,
    renderer: &mut Renderer,
    text: &mut TextSystem,
    theme: &Theme,
) -> Vec<u8> {
    let metrics = UiMetrics::new(1.0);
    let scene = ghost_scene(text, theme, metrics, "tab");
    renderer.sync_atlas(gpu, text.atlas_mut());
    let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
    render_in_format(gpu, renderer, &scene, clear, OFFSCREEN_FORMAT).expect("ghost render")
}

fn hit_cache(text: &mut TextSystem) -> bool {
    let mut missed = false;
    let region = text.image(TEST_KEY, || {
        missed = true;
        vec![255; 64]
    });
    assert!(region.is_some());
    !missed
}

#[test]
fn ghost_scene_renders_after_fresh_gpu_sync_before_paint() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let theme = Theme::new(Appearance::Dark);
    assert!(text.image(TEST_KEY, || vec![255; 64]).is_some());
    let gpu = Gpu::headless().expect("gpu");
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    let before = render_ghost(&gpu, &mut renderer, &mut text, &theme);
    assert!(!text.end_frame());
    assert!(
        before
            .chunks(4)
            .any(|px| px[0] > 150 && px[1] > 150 && px[2] > 150)
    );
    gpu.device.destroy();
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());

    let gpu = Gpu::headless().expect("fresh gpu");
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    text.atlas_mut().invalidate();
    assert!(Running::sync_fresh(true, &mut text, None));

    let after = render_ghost(&gpu, &mut renderer, &mut text, &theme);
    assert_eq!(after.len(), before.len());
    let differing = before
        .chunks_exact(4)
        .zip(after.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(differing, 0, "{differing} ghost pixels differed");
    assert!(!hit_cache(&mut text));
    assert!(!Running::sync_fresh(false, &mut text, None));
    assert!(hit_cache(&mut text));
}

#[test]
fn sync_fresh_resets_picture_uploaded_for_gpu_recovery() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let mut picture = PictureView::new(1, Arc::from(&[][..]), (200, 100));
    picture.uploaded = Some((200, 100));
    assert!(Running::sync_fresh(true, &mut text, Some(&mut picture)));
    assert_eq!(picture.uploaded, None);
}
