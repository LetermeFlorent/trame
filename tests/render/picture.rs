use trame::geometry::Rect;
use trame::render::{Atlas, AtlasKind, Gpu, MAX_COLOR_ATLAS_SIZE, Renderer, Scene, render_to_rgba};
use trame::text::{DEFAULT_LINE_HEIGHT, ImageKey, TextSystem};
use trame::theme::Rgba;
use trame::ui::terminal::preview_rect;

#[test]
fn atlas_image_renders_non_square_pixels() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let mut text = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let mut renderer = Renderer::new(&gpu, trame::render::OFFSCREEN_FORMAT, text.atlas_mut());
    let key = ImageKey {
        id: 42,
        width: 8,
        height: 4,
    };
    let region = text
        .image(key, || [255, 0, 0, 255].repeat(8 * 4))
        .expect("atlas room");
    assert!(!text.end_frame());
    renderer.sync_atlas(&gpu, text.atlas_mut());
    let mut scene = Scene::new(16.0, 12.0);
    scene.glyph(region, 2.0, 3.0, Rgba::WHITE);
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let pixels = render_to_rgba(&gpu, &mut renderer, &scene, clear).expect("render");
    let at = |x: usize, y: usize| -> &[u8] {
        let offset = (y * 16 + x) * 4;
        &pixels[offset..offset + 4]
    };
    assert_eq!(at(5, 5), &[255, 0, 0, 255]);
    assert_eq!(at(0, 0), &[0, 0, 0, 255]);
}

#[test]
fn picture_quad_reads_uploaded_texture() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let mut renderer = Renderer::new(&gpu, trame::render::OFFSCREEN_FORMAT, &mut Atlas::new());
    let pixels = [20, 140, 250, 255].repeat(200 * 100);
    renderer.set_picture(&gpu, Some((200, 100, &pixels)));
    let mut scene = Scene::new(220.0, 140.0);
    scene.picture(Rect::new(10.0, 20.0, 200.0, 100.0));
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let out = render_to_rgba(&gpu, &mut renderer, &scene, clear).expect("render");
    let at = |x: usize, y: usize| -> &[u8] {
        let offset = (y * 220 + x) * 4;
        &out[offset..offset + 4]
    };
    assert_eq!(at(110, 70), &[20, 140, 250, 255]);
    assert_eq!(at(0, 0), &[0, 0, 0, 255]);
}

#[test]
fn max_preview_rect_fits_inside_color_atlas() {
    let rect = preview_rect(Rect::new(0.0, 0.0, 3000.0, 3000.0), 3000, 3000);
    let (w, h) = (rect.width as u32, rect.height as u32);
    assert!(w < MAX_COLOR_ATLAS_SIZE && h < MAX_COLOR_ATLAS_SIZE);
    let mut atlas = Atlas::new();
    let region = atlas.insert(AtlasKind::Color, w, h, vec![255; (w * h * 4) as usize]);
    assert!(region.is_some());
}

#[test]
fn picture_texture_renders_again_on_fresh_gpu_after_reupload() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let mut renderer = Renderer::new(&gpu, trame::render::OFFSCREEN_FORMAT, &mut Atlas::new());
    let pixels = [20, 140, 250, 255].repeat(40 * 20);
    renderer.set_picture(&gpu, Some((40, 20, &pixels)));
    let mut scene = Scene::new(60.0, 40.0);
    scene.picture(Rect::new(10.0, 10.0, 40.0, 20.0));
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let first = render_to_rgba(&gpu, &mut renderer, &scene, clear).expect("first render");
    gpu.device.destroy();
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());

    let fresh_gpu = Gpu::headless().expect("fresh gpu");
    let mut fresh_renderer = Renderer::new(
        &fresh_gpu,
        trame::render::OFFSCREEN_FORMAT,
        &mut Atlas::new(),
    );
    fresh_renderer.set_picture(&fresh_gpu, Some((40, 20, &pixels)));
    let second =
        render_to_rgba(&fresh_gpu, &mut fresh_renderer, &scene, clear).expect("second render");
    assert_eq!(first, second);
}

#[test]
fn set_picture_rejects_oversized_dimensions_without_panicking() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let mut renderer = Renderer::new(&gpu, trame::render::OFFSCREEN_FORMAT, &mut Atlas::new());
    let small = [20, 140, 250, 255].repeat(4 * 4);
    renderer.set_picture(&gpu, Some((4, 4, &small)));

    let max = gpu.device.limits().max_texture_dimension_2d;
    let oversized_side = max.saturating_add(1);
    let oversized = vec![255u8; oversized_side as usize * 4];
    renderer.set_picture(&gpu, Some((oversized_side, 1, &oversized)));
    renderer.set_picture(&gpu, Some((1, oversized_side, &oversized)));

    let mut scene = Scene::new(16.0, 16.0);
    scene.picture(Rect::new(0.0, 0.0, 4.0, 4.0));
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let out = render_to_rgba(&gpu, &mut renderer, &scene, clear).expect("render");
    assert_eq!(&out[..4], &[0, 0, 0, 255]);
}

#[test]
fn set_picture_none_clears_1x1_texture() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let mut renderer = Renderer::new(&gpu, trame::render::OFFSCREEN_FORMAT, &mut Atlas::new());
    renderer.set_picture(&gpu, Some((1, 1, &[255, 0, 0, 255])));
    renderer.set_picture(&gpu, None);
    let mut scene = Scene::new(8.0, 8.0);
    scene.picture(Rect::new(0.0, 0.0, 4.0, 4.0));
    let clear = Rgba::new(0.0, 0.0, 0.0, 1.0);
    let out = render_to_rgba(&gpu, &mut renderer, &scene, clear).expect("render");
    assert_eq!(&out[..4], &[0, 0, 0, 255]);
}
