use trame::files::fit;

use crate::solid;

const RED: [u8; 4] = [255, 0, 0, 255];
const CLEAR: [u8; 4] = [0; 4];

fn pixel(pixels: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * size + x) * 4) as usize;
    pixels[at..at + 4].try_into().expect("four channels")
}

#[test]
fn a_square_image_of_the_right_size_is_kept_as_it_is() {
    let pixels = solid(4, 4, RED);
    assert_eq!(fit(&pixels, 4, 4, 4), pixels);
}

#[test]
fn a_wide_image_is_centred_between_clear_bands() {
    let out = fit(&solid(4, 2, RED), 4, 2, 4);
    assert_eq!(out.len(), 4 * 4 * 4);
    for x in 0..4 {
        assert_eq!(pixel(&out, 4, x, 0), CLEAR);
        assert_eq!(pixel(&out, 4, x, 1), RED);
        assert_eq!(pixel(&out, 4, x, 2), RED);
        assert_eq!(pixel(&out, 4, x, 3), CLEAR);
    }
}

#[test]
fn shrinking_averages_colour_by_coverage() {
    let pixels = [[255, 255, 255, 255], CLEAR, CLEAR, CLEAR].concat();
    assert_eq!(fit(&pixels, 2, 2, 1), vec![255, 255, 255, 64]);
    let half = [[0, 0, 255, 255], [255, 0, 0, 255]].concat();
    assert_eq!(fit(&half, 2, 1, 1), vec![128, 0, 128, 255]);
}

#[test]
fn growing_repeats_the_nearest_pixel() {
    let pixels = [RED, [0, 0, 255, 255]].concat();
    let out = fit(&pixels, 2, 1, 4);
    assert_eq!(pixel(&out, 4, 0, 1), RED);
    assert_eq!(pixel(&out, 4, 1, 2), RED);
    assert_eq!(pixel(&out, 4, 3, 2), [0, 0, 255, 255]);
    assert_eq!(pixel(&out, 4, 0, 0), CLEAR);
}
