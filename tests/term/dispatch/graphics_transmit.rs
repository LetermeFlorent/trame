use png::{BitDepth, ColorType, Encoder};
use trame::term::Terminal;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let pixels = vec![200; (width * height * 4) as usize];
    writer.write_image_data(&pixels).unwrap();
    writer.finish().unwrap();
    bytes
}

fn base64(bytes: &[u8]) -> String {
    let mut text = String::new();
    for group in bytes.chunks(3) {
        let value = group.iter().enumerate().fold(0u32, |value, (at, &byte)| {
            value | u32::from(byte) << (16 - 8 * at)
        });
        for at in 0..4 {
            let sextet = (value >> (18 - 6 * at)) & 0x3f;
            let present = at <= group.len();
            text.push(if present {
                char::from(ALPHABET[sextet as usize])
            } else {
                '='
            });
        }
    }
    text
}

fn send_in_chunks(term: &mut Terminal, id: u32, encoded: &str) {
    let parts = [&encoded[..8], &encoded[8..16], &encoded[16..]];
    term.feed(format!("\x1b_Ga=t,f=100,i={id},m=1;{}\x1b\\", parts[0]).as_bytes());
    term.feed(format!("\x1b_Gm=1;{}\x1b\\", parts[1]).as_bytes());
    term.feed(format!("\x1b_Gm=0;{}\x1b\\", parts[2]).as_bytes());
}

#[test]
fn a_png_sent_in_three_chunks_is_stored() {
    let mut term = Terminal::new(5, 10, 0);
    send_in_chunks(&mut term, 9, &base64(&png(3, 2)));
    let image = term.screen().graphics().get(9).unwrap();
    assert_eq!((image.width, image.height), (3, 2));
    assert_eq!(image.pixels.len(), 3 * 2 * 4);
    assert_eq!(term.screen_mut().take_responses(), b"\x1b_Gi=9;OK\x1b\\");
}

#[test]
fn a_new_command_drops_the_pending_stream() {
    let mut term = Terminal::new(5, 10, 0);
    let encoded = base64(&png(3, 2));
    term.feed(format!("\x1b_Ga=t,f=100,i=9,m=1;{}\x1b\\", &encoded[..8]).as_bytes());
    term.feed(b"\x1b_Ga=q,i=4,s=1,v=1,f=24;AAAA\x1b\\");
    term.feed(format!("\x1b_Gm=0;{}\x1b\\", &encoded[8..]).as_bytes());
    assert!(term.screen().graphics().get(9).is_none());
    assert_eq!(term.screen_mut().take_responses(), b"\x1b_Gi=4;OK\x1b\\");
}

#[test]
fn a_full_reset_drops_the_pending_stream_but_keeps_images() {
    let mut term = Terminal::new(5, 10, 0);
    let encoded = base64(&png(3, 2));
    term.feed(format!("\x1b_Ga=t,f=100,i=9;{encoded}\x1b\\").as_bytes());
    term.feed(format!("\x1b_Ga=t,f=100,i=8,m=1;{}\x1b\\", &encoded[..8]).as_bytes());
    term.feed(b"\x1bc");
    assert!(term.screen().graphics().pending.is_none());
    assert!(term.screen().graphics().get(9).is_some());
}
