use trame::session::pty::PtySize;

#[test]
fn a_new_size_has_no_pixels() {
    let size = PtySize::new(24, 80);
    assert_eq!((size.pixel_width, size.pixel_height), (0, 0));
}

#[test]
fn pixels_are_added_and_clamped() {
    let size = PtySize::new(24, 80).with_pixels(720, 70_000);
    assert_eq!((size.rows, size.cols), (24, 80));
    assert_eq!((size.pixel_width, size.pixel_height), (720, u16::MAX));
}
