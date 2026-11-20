use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, ImageEncoder};

use super::super::decode::handle_picture;
use super::super::decoded::DecodedCache;
use super::super::source::{PictureJob, SlotTarget};
use super::super::worker::SharedState;
use super::super::*;
use crate::files::encode_png;
use crate::text::DEFAULT_LINE_HEIGHT;

fn jpeg_4000x3000() -> Arc<[u8]> {
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
    Arc::from(out.into_inner())
}

#[test]
fn overlay_picture_decodes_off_thread_coalesces_resize_and_wakes() {
    let mut system = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("monospace font");
    let woken = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&woken);
    system.set_image_waker(Arc::new(move || flag.store(true, Ordering::Release)));
    let bytes = jpeg_4000x3000();

    for size in [(800, 600), (640, 480), (400, 300)] {
        system.request_picture(1, size, Arc::clone(&bytes));
    }
    assert!(system.images.picture.is_none());
    assert!(system.images.worker.has_slot_job(SlotTarget::Picture, 1));

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut got = None;
    while Instant::now() < deadline {
        if let Some(res) = system.take_picture(1) {
            got = Some(res);
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let wake_deadline = Instant::now() + Duration::from_secs(1);
    while !woken.load(Ordering::Acquire) && Instant::now() < wake_deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(woken.load(Ordering::Acquire));
    let decoded = got.flatten().expect("decoded overlay picture");
    assert_eq!((decoded.width, decoded.height), (400, 300));
}

#[test]
fn ready_image_cache_evicts_oldest_fifo_instead_of_clearing_all() {
    let mut cache = cache::ImageCache::default();
    for id in 0..33u64 {
        let key = ImageKey {
            id,
            width: 1,
            height: 1,
        };
        cache.record_ready(key, vec![id as u8; 4]);
    }
    assert_eq!(cache.ready.len(), 32);
    let evicted = ImageKey {
        id: 0,
        width: 1,
        height: 1,
    };
    assert!(!cache.ready.contains_key(&evicted));
    let taken = ImageKey {
        id: 1,
        width: 1,
        height: 1,
    };
    cache.take_ready(taken);
    cache.record_ready(
        ImageKey {
            id: 33,
            width: 1,
            height: 1,
        },
        vec![33; 4],
    );
    assert_eq!(cache.ready.len(), 32);
    assert!(cache.ready.contains_key(&ImageKey {
        id: 2,
        width: 1,
        height: 1,
    }));
}

fn run_discarded(png: &Arc<[u8]>, setup: impl FnOnce(&mut SharedState)) {
    let wakes = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&wakes);
    let shared = (Mutex::new(SharedState::default()), Condvar::new());
    {
        let mut state = shared.0.lock().expect("lock");
        state.waker = Some(Arc::new(move || {
            count.fetch_add(1, Ordering::Relaxed);
        }));
        setup(&mut state);
    }
    let mut cache = DecodedCache::default();
    let job = PictureJob {
        job: 1,
        limit: (4, 4),
        source: ImageSource::Bytes(Arc::clone(png)),
    };
    let wake = handle_picture(&shared, &mut cache, job);
    assert!(wake.is_none());
    assert_eq!(wakes.load(Ordering::Relaxed), 0);
    assert!(shared.0.lock().expect("lock").picture_answer.is_none());
}

#[test]
fn cancelled_and_superseded_picture_jobs_discard_without_waking() {
    let png = Arc::<[u8]>::from(encode_png(4, 4, &[255; 64]).expect("png"));
    run_discarded(&png, |_| {});
    run_discarded(&png, |state| {
        state.picture_job = Some(2);
    });
    run_discarded(&png, |state| {
        state.picture_job = Some(1);
        state.picture = Some(PictureJob {
            job: 1,
            limit: (2, 2),
            source: ImageSource::Bytes(Arc::clone(&png)),
        });
    });
    let mut system = TextSystem::new(14.0, DEFAULT_LINE_HEIGHT).expect("font");
    system.request_picture(1, (4, 4), png);
    assert!(system.images.worker.has_slot_job(SlotTarget::Picture, 1));
    system.cancel_picture();
    assert!(!system.images.worker.has_slot_job(SlotTarget::Picture, 1));
}
