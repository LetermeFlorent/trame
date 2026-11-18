use std::sync::Arc;

use trame::render::{downscale, downscaled_size};
use trame::term::graphics::Image;

fn image(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    Image {
        serial: 1,
        width,
        height,
        pixels: Arc::from(pixels),
    }
}

#[test]
fn an_image_within_the_limit_keeps_its_size() {
    assert_eq!(downscaled_size(800, 600, 8192), (800, 600));
}

#[test]
fn a_wide_image_fits_the_limit_and_keeps_its_ratio() {
    assert_eq!(downscaled_size(10000, 5000, 8192), (8192, 4096));
    assert_eq!(downscaled_size(3000, 9000, 4096), (1365, 4096));
}

#[test]
fn a_thin_image_keeps_at_least_one_pixel() {
    assert_eq!(downscaled_size(10000, 1, 100), (100, 1));
}

#[test]
fn small_images_are_uploaded_as_they_are() {
    let pixels = vec![1, 2, 3, 4];
    let image = image(1, 1, pixels.clone());
    let (size, out) = downscale(&image, 8);
    assert_eq!((size, out.into_owned()), ((1, 1), pixels));
}

#[test]
fn the_box_filter_averages_each_block() {
    let pixels = [[0, 0, 0, 255], [200, 100, 50, 255]].repeat(2).concat();
    let image = image(2, 2, pixels);
    let (size, out) = downscale(&image, 1);
    assert_eq!((size, out.into_owned()), ((1, 1), vec![100, 50, 25, 255]));
}

#[test]
fn short_pixel_data_becomes_a_transparent_pixel() {
    let image = image(4, 4, vec![0; 8]);
    let (size, out) = downscale(&image, 8);
    assert_eq!((size, out.into_owned()), ((1, 1), vec![0; 4]));
}
