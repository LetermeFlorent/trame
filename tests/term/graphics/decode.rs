use miniz_oxide::deflate::compress_to_vec_zlib;
use png::{BitDepth, ColorType, Encoder};
use trame::term::graphics::{Error, MAX_SIDE, decode, parse};

fn png(width: u32, height: u32, color: ColorType, data: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(color);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(data).unwrap();
    writer.finish().unwrap();
    bytes
}

#[test]
fn expands_rgb_to_rgba() {
    let command = parse(b"f=24,s=2,v=1");
    let decoded = decode(&command, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(decoded, Ok((2, 1, vec![1, 2, 3, 255, 4, 5, 6, 255])));
}

#[test]
fn keeps_rgba_as_is() {
    let command = parse(b"f=32,s=1,v=2");
    let pixels = vec![1, 2, 3, 4, 5, 6, 7, 8];
    assert_eq!(decode(&command, pixels.clone()), Ok((1, 2, pixels)));
}

#[test]
fn checks_the_raw_length() {
    assert_eq!(
        decode(&parse(b"f=24,s=2,v=1"), vec![0; 5]),
        Err(Error::Inval)
    );
    assert_eq!(
        decode(&parse(b"f=32,s=1,v=1"), vec![0; 3]),
        Err(Error::Inval)
    );
    assert_eq!(
        decode(&parse(b"f=32,s=0,v=1"), Vec::new()),
        Err(Error::Inval)
    );
    assert_eq!(
        decode(&parse(b"f=7,s=1,v=1"), vec![0; 4]),
        Err(Error::Inval)
    );
}

#[test]
fn rejects_oversized_sides() {
    let control = format!("f=32,s={},v=1", MAX_SIDE + 1);
    assert_eq!(
        decode(&parse(control.as_bytes()), Vec::new()),
        Err(Error::TooBig)
    );
}

#[test]
fn decodes_png_to_rgba() {
    let bytes = png(2, 1, ColorType::Rgb, &[10, 20, 30, 40, 50, 60]);
    let decoded = decode(&parse(b"f=100"), bytes);
    assert_eq!(decoded, Ok((2, 1, vec![10, 20, 30, 255, 40, 50, 60, 255])));
}

#[test]
fn rejects_broken_png() {
    assert_eq!(decode(&parse(b"f=100"), vec![1, 2, 3]), Err(Error::Inval));
}

#[test]
fn inflates_zlib_payloads() {
    let pixels = vec![9, 8, 7, 6, 5, 4, 3, 2];
    let compressed = compress_to_vec_zlib(&pixels, 6);
    let decoded = decode(&parse(b"f=32,s=2,v=1,o=z"), compressed);
    assert_eq!(decoded, Ok((2, 1, pixels)));
}

#[test]
fn rejects_broken_zlib() {
    assert_eq!(
        decode(&parse(b"f=32,s=1,v=1,o=z"), vec![0; 4]),
        Err(Error::Inval)
    );
}
