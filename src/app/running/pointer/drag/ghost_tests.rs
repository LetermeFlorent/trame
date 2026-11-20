#[path = "ghost_tests/overflow.rs"]
mod overflow;

use super::{GhostGraphics, GhostTab, ghost_cursor_pixels, ghost_pixels_with_retry};
use trame::render::{
    Atlas, Gpu, OFFSCREEN_FORMAT, ReadbackError, Renderer, Scene, render_in_format,
};
use trame::text::{DEFAULT_LINE_HEIGHT, ImageKey, TextSystem};
use trame::theme::{Appearance, Rgba, Theme};
use trame::ui::UiMetrics;

const SQUARE_KEY: ImageKey = ImageKey {
    id: 7,
    width: 4,
    height: 4,
};

fn tab(theme: &Theme) -> GhostTab<'_> {
    GhostTab {
        theme,
        metrics: UiMetrics::new(1.0),
        label: "tab",
    }
}

fn draw(fresh: bool, gpu: &Gpu, renderer: &mut Renderer, text: &mut TextSystem) -> Vec<u8> {
    let theme = Theme::new(Appearance::Dark);
    let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
    let (pixels, w, h) = ghost_cursor_pixels(fresh, text, tab(&theme), |s, a| {
        renderer.sync_atlas(gpu, a);
        render_in_format(gpu, renderer, s, clear, OFFSCREEN_FORMAT)
    })
    .expect("ghost");
    assert_eq!(pixels.len(), usize::from(w) * usize::from(h) * 4);
    pixels
}

fn cache_missed(text: &mut TextSystem, key: ImageKey) -> bool {
    let mut missed = false;
    let region = text.image(key, || {
        missed = true;
        vec![255; 64]
    });
    assert!(region.is_some());
    missed
}

#[derive(Default)]
struct StubGhost {
    device: Option<(Gpu, Renderer)>,
    fresh: bool,
    fail_left: usize,
    set_fresh: bool,
    calls: usize,
}

impl StubGhost {
    fn open(atlas: &mut Atlas) -> Self {
        let gpu = Gpu::headless().expect("gpu");
        let renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, atlas);
        Self {
            device: Some((gpu, renderer)),
            set_fresh: true,
            ..Default::default()
        }
    }
}

impl GhostGraphics for StubGhost {
    fn take_fresh(&mut self) -> bool {
        std::mem::take(&mut self.fresh)
    }

    fn render_image(&mut self, scene: &Scene, atlas: &mut Atlas) -> Result<Vec<u8>, ReadbackError> {
        self.calls += 1;
        if self.fail_left > 0 {
            self.fail_left -= 1;
            if let Some((gpu, _)) = self.device.take() {
                gpu.device.destroy();
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                let fresh_gpu = Gpu::headless().expect("fresh gpu");
                let fresh_renderer = Renderer::new(&fresh_gpu, OFFSCREEN_FORMAT, atlas);
                self.device = Some((fresh_gpu, fresh_renderer));
                atlas.invalidate();
            }
            self.fresh = self.set_fresh;
            return Err(ReadbackError::Lost);
        }
        let (gpu, renderer) = self.device.as_mut().expect("gpu");
        renderer.sync_atlas(gpu, atlas);
        let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
        render_in_format(gpu, renderer, scene, clear, OFFSCREEN_FORMAT)
    }
}

#[test]
fn ghost_cursor_syncs_fresh_gpu_before_scene() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    assert!(text.image(SQUARE_KEY, || vec![255; 64]).is_some());
    let gpu = Gpu::headless().expect("gpu");
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    let before = draw(false, &gpu, &mut renderer, &mut text);
    assert!(!cache_missed(&mut text, SQUARE_KEY));
    assert!(!text.end_frame());
    gpu.device.destroy();
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());

    let fresh_gpu = Gpu::headless().expect("fresh gpu");
    let mut fresh_renderer = Renderer::new(&fresh_gpu, OFFSCREEN_FORMAT, text.atlas_mut());
    text.atlas_mut().invalidate();

    let after = draw(true, &fresh_gpu, &mut fresh_renderer, &mut text);
    assert!(before == after, "ghost cursor pixels differed");
    assert!(cache_missed(&mut text, SQUARE_KEY));
    assert!(!text.end_frame());
}

#[test]
fn ghost_cursor_retries_once_when_failed_render_yields_fresh_gpu() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let theme = Theme::new(Appearance::Dark);
    assert!(text.image(SQUARE_KEY, || vec![255; 64]).is_some());
    let mut dev = StubGhost::open(text.atlas_mut());
    let before = ghost_pixels_with_retry(&mut dev, &mut text, tab(&theme)).expect("initial");
    assert!(!cache_missed(&mut text, SQUARE_KEY));
    assert!(!text.end_frame());

    dev.fail_left = 1;
    let after = ghost_pixels_with_retry(&mut dev, &mut text, tab(&theme)).expect("retry");
    assert!(before == after, "retried ghost cursor differed");
    assert_eq!(dev.calls, 3);
    assert!(cache_missed(&mut text, SQUARE_KEY));
    assert!(!text.end_frame());
}

#[test]
fn ghost_cursor_does_not_retry_without_fresh_or_more_than_once() {
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let theme = Theme::new(Appearance::Dark);
    for (set_fresh, expected_calls) in [(false, 1), (true, 2)] {
        let mut dev = StubGhost {
            fail_left: usize::MAX,
            set_fresh,
            ..Default::default()
        };
        let err = ghost_pixels_with_retry(&mut dev, &mut text, tab(&theme));
        assert!(matches!(err, Err(ReadbackError::Lost)));
        assert_eq!(dev.calls, expected_calls);
    }
}
