use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use trame::files::encode_png;
use trame::geometry::Rect;
use trame::render::MAX_COLOR_ATLAS_SIZE;
use trame::term::Terminal;
use trame::ui::terminal::preview_rect;

use super::helpers::{draw, jpeg_4000x3000, osc_image, quad_drawn_rect, wait_images, wait_woken};
use crate::text_system;

#[test]
fn preview_rect_caps_at_max_color_atlas_size_minus_one() {
    let rect = preview_rect(Rect::new(0.0, 0.0, 3000.0, 3000.0), 4000, 3000);
    assert!(rect.width <= (MAX_COLOR_ATLAS_SIZE - 1) as f32);
    assert!(rect.height <= (MAX_COLOR_ATLAS_SIZE - 1) as f32);
}

#[test]
fn inline_4000x3000_jpeg_renders_async_without_blocking() {
    let mut term = Terminal::new(24, 80, 0);
    term.feed(&osc_image(&jpeg_4000x3000()));
    assert_eq!(term.screen().image_count(), 1);
    let mut text = text_system();
    let woken = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&woken);
    text.set_image_waker(Arc::new(move || {
        flag.store(true, std::sync::atomic::Ordering::Release);
    }));

    let frame1 = draw(&mut text, &term);
    assert!(text.has_pending_images());
    assert!(!frame1.quads().iter().any(|quad| quad.params[0] == 3.0));
    assert!(frame1.quads().iter().any(|quad| quad.params[0] == 0.0));

    wait_images(&mut text);
    wait_woken(&woken);

    let frame2 = draw(&mut text, &term);
    assert!(!text.has_pending_images());
    assert!(frame2.quads().iter().any(|quad| quad.params[0] == 3.0));
}

#[test]
fn overwritten_cells_are_never_covered_by_inline_image_preview() {
    let mut term = Terminal::new(10, 40, 0);
    let png = encode_png(64, 32, &[200, 40, 80, 255].repeat(64 * 32)).expect("png");
    term.feed(&osc_image(&png));
    term.feed(b"\x1b[1;1Hhello");
    let mut text = text_system();
    let metrics = text.cell_metrics();
    let overwritten = Rect::new(0.0, 0.0, 5.0 * metrics.width, metrics.height);

    let pending = draw(&mut text, &term);
    wait_images(&mut text);
    let ready = draw(&mut text, &term);

    assert!(ready.quads().iter().any(|quad| quad.params[0] == 3.0));
    for quad in pending.quads().iter().chain(ready.quads()) {
        if quad.params[0] == 3.0 || (quad.params[0] == 0.0 && quad.rect[2] > 50.0) {
            let drawn = quad_drawn_rect(quad.rect, quad.clip);
            assert!(
                drawn.intersect(overwritten).is_empty(),
                "image covered overwritten text: {drawn:?}"
            );
        }
    }
}

#[test]
fn inline_preview_respects_device_max_texture_size() {
    let mut term = Terminal::new(24, 80, 0);
    let png = encode_png(200, 200, &[20, 180, 90, 255].repeat(200 * 200)).expect("png");
    term.feed(&osc_image(&png));
    let mut text = text_system();
    text.atlas_mut().limit_to(100);

    let _ = draw(&mut text, &term);
    wait_images(&mut text);
    let scene = draw(&mut text, &term);

    let color_quad = scene
        .quads()
        .iter()
        .find(|quad| quad.params[0] == 3.0)
        .expect("image quad within 100px limit");
    assert!(color_quad.rect[2] <= 99.0 && color_quad.rect[3] <= 99.0);
}
