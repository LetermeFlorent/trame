use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, ImageEncoder};
use trame::geometry::{Point, Rect};
use trame::render::Scene;
use trame::term::Terminal;
use trame::text::TextSystem;
use trame::theme::{Appearance, Theme};
use trame::ui::terminal::{ScreenCopy, TerminalPaint, draw_terminal};

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(super) fn osc_image(bytes: &[u8]) -> Vec<u8> {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
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
    format!("\x1b]1337;File=inline=1:{out}\x07").into_bytes()
}

pub(super) fn jpeg_4000x3000() -> Vec<u8> {
    let mut rgb = Vec::with_capacity(4000 * 3000 * 3);
    for y in 0..3000u32 {
        for x in 0..4000u32 {
            rgb.push(((x / 16) * 13 + (y / 16) * 7) as u8);
            rgb.push(((x / 8) ^ (y / 8)) as u8);
            rgb.push(((x + y) / 32) as u8);
        }
    }
    let mut out = Cursor::new(Vec::new());
    JpegEncoder::new_with_quality(&mut out, 75)
        .write_image(&rgb, 4000, 3000, ColorType::Rgb8.into())
        .expect("jpeg");
    out.into_inner()
}

pub(super) fn draw(text: &mut TextSystem, term: &Terminal) -> Scene {
    let mut scene = Scene::new(1200.0, 700.0);
    let copy = ScreenCopy::of(term.screen(), 0.0).with_session(1);
    let paint = TerminalPaint {
        screen: &copy,
        origin: Point::new(0.0, 0.0),
        focused: true,
        scroll: 0.0,
        selected: &|_, _| false,
        links: &[],
    };
    draw_terminal(&mut scene, text, &Theme::new(Appearance::Dark), &paint);
    scene
}

pub(super) fn wait_images(text: &mut TextSystem) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while text.has_pending_images() && Instant::now() < deadline {
        if !text.receive_images() {
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    text.receive_images();
    assert!(!text.has_pending_images(), "image worker timed out");
}

pub(super) fn wait_woken(woken: &AtomicBool) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while !woken.load(Ordering::Acquire) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(woken.load(Ordering::Acquire), "image waker did not fire");
}

pub(super) fn quad_drawn_rect(rect: [f32; 4], clip: [f32; 4]) -> Rect {
    let rect = Rect::new(rect[0], rect[1], rect[2], rect[3]);
    let clip = Rect::from_edges(clip[0], clip[1], clip[2], clip[3]);
    rect.intersect(clip)
}
