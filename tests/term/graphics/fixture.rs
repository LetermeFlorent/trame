use trame::term::Terminal;

pub fn send(term: &mut Terminal, keys: &str) {
    term.feed(format!("\x1b_G{keys}\x1b\\").as_bytes());
}

pub fn put(term: &mut Terminal, keys: &str) {
    term.feed(format!("\x1b_Ga=T,f=24,s=1,v=1,q=2,{keys};AAAA\x1b\\").as_bytes());
}

pub fn placed(term: &Terminal, row: usize) -> Vec<(u32, u32)> {
    term.screen()
        .grid()
        .row(row)
        .anchors()
        .iter()
        .map(|anchor| (anchor.image, anchor.placement))
        .collect()
}

pub fn all_placed(term: &Terminal) -> Vec<(u32, u32)> {
    term.screen()
        .grid()
        .anchors()
        .map(|anchor| (anchor.image, anchor.placement))
        .collect()
}

pub fn encoded_png(width: u32, height: u32) -> String {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer
        .write_image_data(&vec![90; (width * height * 4) as usize])
        .unwrap();
    writer.finish().unwrap();
    base64(&bytes)
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::new();
    for group in bytes.chunks(3) {
        let value = group.iter().enumerate().fold(0u32, |value, (at, &byte)| {
            value | u32::from(byte) << (16 - 8 * at)
        });
        for at in 0..4 {
            let sextet = (value >> (18 - 6 * at)) & 0x3f;
            text.push(if at <= group.len() {
                char::from(ALPHABET[sextet as usize])
            } else {
                '='
            });
        }
    }
    text
}
