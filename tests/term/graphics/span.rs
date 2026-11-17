use trame::term::graphics::{Command, Crop, crop, span};

fn sized(cols: u16, rows: u16) -> Command {
    Command {
        cols,
        rows,
        ..Command::default()
    }
}

fn full(w: u32, h: u32) -> Crop {
    Crop { x: 0, y: 0, w, h }
}

#[test]
fn the_crop_defaults_to_the_rest_of_the_image() {
    let mut command = Command::default();
    command.crop.x = 10;
    command.crop.h = 500;
    assert_eq!(
        crop(&command, 40, 30),
        Crop {
            x: 10,
            y: 0,
            w: 30,
            h: 30
        }
    );
}

#[test]
fn the_span_rounds_the_pixel_size_up_to_cells() {
    assert_eq!(span(&sized(0, 0), full(25, 33), (10, 20)), (3, 2));
    assert_eq!(span(&sized(0, 0), full(16, 32), (0, 0)), (2, 2));
}

#[test]
fn the_span_keeps_the_aspect_ratio_when_one_side_is_given() {
    assert_eq!(span(&sized(4, 0), full(100, 100), (10, 20)), (4, 2));
    assert_eq!(span(&sized(0, 3), full(100, 100), (10, 20)), (6, 3));
    assert_eq!(span(&sized(7, 9), full(1, 1), (10, 20)), (7, 9));
}

#[test]
fn the_span_never_collapses() {
    assert_eq!(span(&sized(0, 0), full(0, 0), (10, 20)), (1, 1));
}
