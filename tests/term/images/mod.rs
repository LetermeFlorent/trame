mod limits;

use trame::files::encode_png;
use trame::term::{Flags, Terminal};

use crate::{cursor, terminal};

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(super) fn b64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
        let b2 = u32::from(*chunk.get(2).unwrap_or(&0));
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

pub(super) fn png(width: usize, height: usize) -> Vec<u8> {
    encode_png(width, height, &[200, 40, 80, 255].repeat(width * height)).expect("png")
}

pub(super) fn osc(args: &str, payload: &[u8]) -> Vec<u8> {
    format!("\x1b]1337;File={args}:{}\x07", b64(payload)).into_bytes()
}

pub(super) fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

pub(super) fn patched_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = png(1, 1);
    bytes[16..20].copy_from_slice(&width.to_be_bytes());
    bytes[20..24].copy_from_slice(&height.to_be_bytes());
    let crc = crc32(&bytes[12..29]);
    bytes[29..33].copy_from_slice(&crc.to_be_bytes());
    bytes
}

pub(super) fn has_image(term: &Terminal) -> bool {
    let grid = term.screen().grid();
    (grid.first_line()..grid.end_line())
        .filter_map(|line| grid.line(line))
        .flat_map(|row| row.cells())
        .any(|cell| cell.flags.contains(Flags::IMAGE))
}

#[test]
fn inline_png_fills_block_with_row_major_tiles_and_moves_cursor_below() {
    let mut term = terminal(24, 80);
    term.feed(&osc("inline=1", &png(64, 32)));
    assert_eq!(cursor(&term), (8, 0));
    let grid = term.screen().grid();
    let id = grid.row(0).cell(0).image_id().expect("image id");
    for row in 0..8 {
        for col in 0..32 {
            let cell = grid.row(row).cell(col);
            assert!(cell.flags.contains(Flags::IMAGE));
            assert_eq!(cell.image_id(), Some(id));
            assert_eq!(cell.image_tile(), Some((row as u16, col as u16)));
        }
        assert!(!grid.row(row).cell(32).flags.contains(Flags::IMAGE));
    }
    assert!(!grid.row(8).cell(0).flags.contains(Flags::IMAGE));
}

#[test]
fn non_inline_and_invalid_payloads_leave_grid_unchanged() {
    for payload in [
        osc("inline=0", &png(4, 4)),
        osc("size=4", &png(4, 4)),
        b"\x1b]1337;File=inline=1:!!!\x07".to_vec(),
        osc("inline=1", b"not an image"),
        osc("inline=1", &png(4, 4)[..16]),
        osc("inline=1", &patched_png(100_000, 100_000)),
    ] {
        let mut term = terminal(6, 20);
        term.feed(b"a");
        term.feed(&payload);
        assert_eq!(cursor(&term), (0, 1));
        assert!(!has_image(&term));
        assert_eq!(term.screen().grid().row(0).text(), "a");
    }
}

#[test]
fn narrow_and_small_grids_cap_preview_dimensions() {
    let mut narrow = terminal(24, 10);
    narrow.feed(&osc("inline=1", &png(64, 32)));
    let grid = narrow.screen().grid();
    assert!(grid.row(0).cell(9).flags.contains(Flags::IMAGE));
    assert!(!grid.row(0).cell(10).flags.contains(Flags::IMAGE));

    let mut small = terminal(4, 40);
    small.feed(&osc("inline=1", &png(64, 32)));
    assert_eq!(cursor(&small), (3, 0));
    assert!(
        small
            .screen()
            .grid()
            .row(2)
            .cell(0)
            .flags
            .contains(Flags::IMAGE)
    );
    assert!(
        !small
            .screen()
            .grid()
            .row(3)
            .cell(0)
            .flags
            .contains(Flags::IMAGE)
    );
}
