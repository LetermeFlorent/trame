use trame::geometry::Rect;
use trame::render::{Atlas, Gpu, Renderer, Scene, render_in_format};
use trame::theme::Rgba;

const SIDE: f32 = 8.0;

fn red_square(gpu: &Gpu, format: wgpu::TextureFormat) -> Vec<u8> {
    let mut renderer = Renderer::new(gpu, format, &Atlas::new());
    let mut scene = Scene::new(SIDE, SIDE);
    scene.fill(
        Rect::new(0.0, 0.0, SIDE / 2.0, SIDE),
        Rgba::new(1.0, 0.0, 0.0, 1.0),
        0.0,
    );
    let clear = Rgba::new(0.0, 0.0, 0.0, 0.0);
    render_in_format(gpu, &mut renderer, &scene, clear, format).expect("the pixels are read")
}

#[test]
fn every_format_comes_back_in_rgba_order() {
    let gpu = Gpu::headless().expect("a graphics adapter");
    for format in [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Bgra8Unorm,
    ] {
        let pixels = red_square(&gpu, format);
        assert_eq!(pixels.len(), (SIDE * SIDE * 4.0) as usize);
        assert_eq!(&pixels[..4], &[255, 0, 0, 255], "{format:?}");
        let right = (SIDE as usize - 1) * 4;
        assert_eq!(&pixels[right..right + 4], &[0, 0, 0, 0], "{format:?}");
    }
}
