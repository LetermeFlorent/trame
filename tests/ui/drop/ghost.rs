use trame::theme::{Appearance, Theme};
use trame::ui::UiMetrics;
use trame::ui::drop::{ghost_scene, stamp_arrow, straight_alpha};

use crate::text_system;

const ARROW_WIDTH: usize = 12;
const ARROW_HEIGHT: usize = 19;

fn pixel(pixels: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let at = (y * width + x) * 4;
    pixels[at..at + 4].try_into().expect("four channels")
}

#[test]
fn the_ghost_image_holds_the_label_and_leaves_room_for_the_arrow() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let metrics = UiMetrics::new(1.0);
    let short = ghost_scene(&mut text, &theme, metrics, "a");
    let long = ghost_scene(&mut text, &theme, metrics, "a much longer tab name");
    assert!(long.width() > short.width());
    assert_eq!(long.height(), short.height());
    assert!(short.width() >= ARROW_WIDTH as f32 && short.height() >= ARROW_HEIGHT as f32);
    assert!(long.height() >= UiMetrics::TAB_BAR);
    let bubble = long
        .quads()
        .iter()
        .find(|quad| quad.color == theme.border_strong.to_array())
        .expect("an outline");
    assert!(bubble.rect[0] > 0.0 && bubble.rect[1] > 0.0);
    assert!(bubble.rect[0] + bubble.rect[2] <= long.width());
    assert!(bubble.rect[1] + bubble.rect[3] <= long.height());
}

#[test]
fn a_very_long_label_is_cut_to_the_widest_tab() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Light);
    let metrics = UiMetrics::new(1.0);
    let scene = ghost_scene(&mut text, &theme, metrics, &"w".repeat(400));
    assert!(scene.width() < UiMetrics::TAB_MAX * 2.0);
    assert!(scene.width() <= f32::from(u16::MAX));
}

#[test]
fn premultiplied_pixels_become_straight() {
    let mut pixels = vec![
        64, 32, 0, 128, 10, 20, 30, 0, 200, 100, 50, 255, 255, 255, 255, 1,
    ];
    straight_alpha(&mut pixels);
    assert_eq!(&pixels[0..4], &[127, 63, 0, 128]);
    assert_eq!(&pixels[4..8], &[0, 0, 0, 0]);
    assert_eq!(&pixels[8..12], &[200, 100, 50, 255]);
    assert_eq!(&pixels[12..16], &[255, 255, 255, 1]);
}

#[test]
fn the_arrow_tip_sits_on_the_first_pixel() {
    let width = 20;
    let height = 24;
    let mut pixels = vec![0; width * height * 4];
    stamp_arrow(&mut pixels, width as u32, UiMetrics::new(1.0));
    assert_eq!(pixel(&pixels, width, 0, 0), [0, 0, 0, 255]);
    assert_eq!(pixel(&pixels, width, 1, 0), [0, 0, 0, 0]);
    assert_eq!(pixel(&pixels, width, 1, 2), [255, 255, 255, 255]);
    assert_eq!(pixel(&pixels, width, ARROW_WIDTH, 5), [0, 0, 0, 0]);
    assert_eq!(pixel(&pixels, width, 0, ARROW_HEIGHT), [0, 0, 0, 0]);
}

#[test]
fn the_arrow_grows_with_the_screen_scale_and_stays_inside_the_image() {
    let width = 30;
    let mut pixels = vec![0; width * 50 * 4];
    stamp_arrow(&mut pixels, width as u32, UiMetrics::new(2.0));
    assert_eq!(pixel(&pixels, width, 1, 1), [0, 0, 0, 255]);
    assert_eq!(pixel(&pixels, width, 2, 4), [255, 255, 255, 255]);
    let mut tiny = vec![0; 4 * 4 * 4];
    stamp_arrow(&mut tiny, 4, UiMetrics::new(2.0));
    assert_eq!(pixel(&tiny, 4, 0, 0), [0, 0, 0, 255]);
}
