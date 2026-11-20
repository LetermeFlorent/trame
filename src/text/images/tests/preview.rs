use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::super::*;
use crate::files::encode_png;
use crate::text::DEFAULT_LINE_HEIGHT;

const WEBP_1X1: &[u8] = &[
    82, 73, 70, 70, 30, 0, 0, 0, 87, 69, 66, 80, 86, 80, 56, 76, 17, 0, 0, 0, 47, 0, 0, 0, 0, 7,
    80, 198, 82, 84, 191, 255, 129, 136, 232, 127, 0, 0,
];

fn wait_slot(
    system: &mut TextSystem,
    mut poll: impl FnMut(&mut TextSystem) -> Option<Option<DecodedPicture>>,
) -> DecodedPicture {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Some(res) = poll(system) {
            return res.expect("decoded image");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("worker timed out");
}

#[test]
fn hover_preview_decodes_local_webp_off_thread_and_refreshes_on_change() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let path: PathBuf = std::env::temp_dir().join(format!("trame-test-hover-{stamp}.webp"));
    std::fs::write(&path, WEBP_1X1).expect("write webp");
    let mut system = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    let woken = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&woken);
    system.set_image_waker(Arc::new(move || flag.store(true, Ordering::Release)));

    system.request_preview(7, (120, 90), path.clone());
    let decoded = wait_slot(&mut system, |s| s.take_preview(7));
    assert_eq!((decoded.width, decoded.height), (1, 1));
    assert_eq!(decoded.src_size, (1, 1));
    assert_eq!(&decoded.pixels, &[20, 140, 250, 255]);
    assert!(woken.load(Ordering::Acquire));

    let png = encode_png(2, 2, &[9, 8, 7, 255].repeat(4)).expect("png");
    std::fs::write(&path, &png).expect("overwrite");
    system.request_picture(8, (100, 100), path.clone());
    let big = wait_slot(&mut system, |s| s.take_picture(8));
    let _ = std::fs::remove_file(&path);
    assert_eq!(big.src_size, (2, 2));
}

#[test]
fn hover_image_rejects_mismatched_pixel_buffer_len() {
    let mut system = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    assert!(system.hover_image(1, 2, 2, &[0; 4]).is_none());
    assert!(system.hover_image(2, 2, 2, &[0; 20]).is_none());
    assert!(system.hover_image(3, u32::MAX, u32::MAX, &[0; 4]).is_none());
    assert!(system.hover_image(4, 2, 2, &[255; 16]).is_some());
}
