use std::sync::Arc;

use trame::geometry::Rect;
use trame::render::{Atlas, Gpu, OFFSCREEN_FORMAT, Renderer, Scene, render_to_rgba};
use trame::term::graphics::Image;
use trame::theme::Rgba;

const SIDE: f32 = 8.0;
const RED: [u8; 4] = [255, 0, 0, 255];

fn red_image() -> Image {
    Image {
        serial: 3,
        width: 2,
        height: 2,
        pixels: Arc::from(RED.repeat(4)),
    }
}

fn scene() -> Scene {
    let mut scene = Scene::new(SIDE, SIDE);
    scene.image(
        &red_image(),
        Rect::new(0.0, 0.0, SIDE / 2.0, SIDE),
        [0.0, 0.0, 1.0, 1.0],
    );
    scene
}

#[test]
fn an_image_quad_is_recorded_once_for_upload() {
    let mut scene = scene();
    scene.image(&red_image(), Rect::new(4.0, 0.0, 4.0, 4.0), [0.0; 4]);
    let quads: Vec<u32> = scene.images().iter().map(|draw| draw.quad).collect();
    assert_eq!(quads, vec![0, 1]);
    assert_eq!(scene.image_uploads().len(), 1);
    assert_eq!(scene.quads()[0].params[0], 6.0);
    scene.reset(SIDE, SIDE);
    assert!(scene.images().is_empty() && scene.image_uploads().is_empty());
}

#[test]
fn a_clipped_image_is_not_recorded() {
    let mut scene = Scene::new(SIDE, SIDE);
    scene.image(&red_image(), Rect::new(20.0, 20.0, 4.0, 4.0), [0.0; 4]);
    assert!(scene.quads().is_empty() && scene.images().is_empty());
}

#[test]
fn an_image_tile_renders_its_pixels() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    let mut renderer = Renderer::new(&gpu, OFFSCREEN_FORMAT, &mut Atlas::new());
    let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
    for _ in 0..2 {
        let pixels = render_to_rgba(&gpu, &mut renderer, &scene(), clear).expect("the pixels");
        assert_eq!(&pixels[..4], &RED);
        let right = (SIDE as usize - 1) * 4;
        assert_eq!(&pixels[right..right + 4], &[0, 0, 0, 0]);
    }
}
