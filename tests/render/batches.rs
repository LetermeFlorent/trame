use trame::render::{ImageDraw, batches};

fn image(quad: u32, serial: u64) -> ImageDraw {
    ImageDraw { quad, serial }
}

#[test]
fn a_scene_without_images_is_one_batch() {
    assert_eq!(batches(5, &[]), vec![(0..5, None)]);
}

#[test]
fn an_empty_scene_has_no_batch() {
    assert!(batches(0, &[]).is_empty());
}

#[test]
fn an_image_in_the_middle_splits_the_quads() {
    assert_eq!(
        batches(6, &[image(2, 9)]),
        vec![(0..2, None), (2..3, Some(9)), (3..6, None)]
    );
}

#[test]
fn images_at_both_ends_leave_no_empty_batch() {
    assert_eq!(
        batches(3, &[image(0, 1), image(2, 1)]),
        vec![(0..1, Some(1)), (1..2, None), (2..3, Some(1))]
    );
}

#[test]
fn consecutive_tiles_of_one_image_share_a_batch() {
    assert_eq!(
        batches(4, &[image(1, 7), image(2, 7), image(3, 8)]),
        vec![(0..1, None), (1..3, Some(7)), (3..4, Some(8))]
    );
}
