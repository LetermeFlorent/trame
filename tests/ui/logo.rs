use trame::geometry::Rect;
use trame::render::Scene;
use trame::theme::Rgba;
use trame::ui::chrome::logo::{SIZE, draw_logo, draw_weaving, pixels, window_icon};

const BYTES_PER_PIXEL: usize = 4;
const CENTER: u32 = SIZE / 2;

fn pixel(x: u32, y: u32) -> &'static [u8] {
    let start = (y * SIZE + x) as usize * BYTES_PER_PIXEL;
    &pixels()[start..start + BYTES_PER_PIXEL]
}

#[test]
fn embedded_pixels_cover_the_whole_square() {
    assert_eq!(pixels().len(), (SIZE * SIZE) as usize * BYTES_PER_PIXEL);
}

#[test]
fn corners_are_fully_transparent() {
    let last = SIZE - 1;
    for (x, y) in [(0, 0), (last, 0), (0, last), (last, last)] {
        assert_eq!(pixel(x, y)[3], 0, "corner ({x}, {y})");
    }
}

#[test]
fn a_thread_pixel_is_opaque_blue() {
    let thread_x = SIZE / 3;
    assert_eq!(pixel(thread_x, CENTER), [0x3b, 0x7d, 0xdd, 0xff]);
}

#[test]
fn winit_accepts_the_pixels() {
    assert!(window_icon().is_some());
}

#[test]
fn the_drawn_logo_stays_inside_its_square() {
    let square = Rect::new(20.0, 30.0, 120.0, 120.0);
    let mut scene = Scene::new(200.0, 200.0);
    draw_logo(&mut scene, square);
    assert_eq!(scene.quads().len(), 8);
    for quad in scene.quads() {
        let [x, y, width, height] = quad.rect;
        assert!(x >= square.x && y >= square.y, "{:?}", quad.rect);
        assert!(x + width <= square.right() && y + height <= square.bottom());
    }
}

#[test]
fn the_weaving_logo_threads_in_and_out_over_a_faint_copy() {
    let square = Rect::new(0.0, 0.0, 24.0, 24.0);
    let threads = |phase: f32| {
        let mut scene = Scene::new(40.0, 40.0);
        draw_weaving(&mut scene, square, Rgba::WHITE, phase);
        let quads = scene.quads();
        let bright = quads.iter().filter(|quad| quad.color[3] >= 1.0).count();
        (quads.len() - bright, bright)
    };
    assert_eq!(threads(0.0), (8, 0));
    assert_eq!(threads(0.48), (8, 8));
    assert_eq!(threads(0.97), (8, 0));
}
