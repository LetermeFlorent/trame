use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread;

use gtk::{gdk, glib};
use trame::files::pictures::{RgbaImage, decode, fit_into, image_limits, read_image_bytes};
use trame::text::ImageSource;

const BYTES_PER_PIXEL: usize = 4;
const WORKER_NAME: &str = "gtk-image";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PreviewKey {
    pub session_id: u64,
    pub image_id: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadyPixels {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub enum WorkerJob {
    Inline(PreviewKey, (u32, u32), Arc<[u8]>),
    Overlay(u64, (u32, u32), ImageSource),
    Hover(u64, (u32, u32), PathBuf),
}

pub enum WorkerAnswer {
    Inline(PreviewKey, Option<ReadyPixels>),
    Overlay(u64, Option<ReadyPixels>),
    Hover(u64, Option<ReadyPixels>),
}

#[derive(Clone)]
pub struct ImageWorker {
    sender: Sender<WorkerJob>,
}

impl ImageWorker {
    pub fn spawn(mut handle: impl FnMut(WorkerAnswer) -> bool + 'static) -> Self {
        let (job_tx, job_rx) = mpsc::channel();
        let (ans_tx, ans_rx) = async_channel::unbounded();
        let _ = thread::Builder::new()
            .name(WORKER_NAME.into())
            .spawn(move || {
                while let Ok(job) = job_rx.recv() {
                    if ans_tx.send_blocking(decode_job(job)).is_err() {
                        break;
                    }
                }
            });
        glib::spawn_future_local(async move {
            while let Ok(answer) = ans_rx.recv().await {
                if !handle(answer) {
                    break;
                }
            }
        });
        Self { sender: job_tx }
    }

    pub fn send(&self, job: WorkerJob) {
        let _ = self.sender.send(job);
    }
}

pub(crate) fn decode_job(job: WorkerJob) -> WorkerAnswer {
    match job {
        WorkerJob::Inline(key, limit, bytes) => {
            let image = decode(&bytes, image_limits())
                .ok()
                .and_then(|img| scaled_pixels(&img, limit, false))
                .filter(|ready| (ready.width, ready.height) == (key.width, key.height));
            WorkerAnswer::Inline(key, image)
        }
        WorkerJob::Overlay(job, limit, source) => {
            let image = decode_source(&source).and_then(|img| scaled_pixels(&img, limit, true));
            WorkerAnswer::Overlay(job, image)
        }
        WorkerJob::Hover(job, limit, path) => {
            let src = ImageSource::from(path);
            let image = decode_source(&src).and_then(|img| scaled_pixels(&img, limit, true));
            WorkerAnswer::Hover(job, image)
        }
    }
}

fn decode_source(source: &ImageSource) -> Option<RgbaImage> {
    match source {
        ImageSource::Bytes(bytes) => decode(bytes, image_limits()).ok(),
        ImageSource::Path(path) => read_image_bytes(path)
            .and_then(|bytes| decode(&bytes, image_limits()))
            .ok(),
    }
}

fn scaled_pixels(decoded: &RgbaImage, limit: (u32, u32), cap_to_src: bool) -> Option<ReadyPixels> {
    let (src_w, src_h) = decoded.dimensions();
    let max = if cap_to_src {
        (limit.0.min(src_w), limit.1.min(src_h))
    } else {
        limit
    };
    let (pixels, width, height) = fit_into(decoded, max.0, max.1);
    (width > 0 && height > 0).then_some(ReadyPixels {
        width,
        height,
        pixels,
    })
}

pub fn memory_texture(ready: ReadyPixels) -> Option<gdk::MemoryTexture> {
    let stride = (ready.width as usize).checked_mul(BYTES_PER_PIXEL)?;
    let expected = stride.checked_mul(ready.height as usize)?;
    if ready.width == 0 || ready.height == 0 || ready.pixels.len() != expected {
        return None;
    }
    let bytes = glib::Bytes::from_owned(ready.pixels);
    let (w, h) = (ready.width as i32, ready.height as i32);
    Some(gdk::MemoryTexture::new(
        w,
        h,
        gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        stride,
    ))
}
