use std::sync::{Condvar, Mutex, PoisonError};

use super::decoded::DecodedCache;
use super::source::{
    DecodedPicture, ImageSource, InlineAnswer, InlineJob, PictureAnswer, PictureJob, SlotTarget,
};
use crate::files::pictures::fit_into;

enum Task {
    Slot(SlotTarget, PictureJob),
    Inline(InlineJob),
}

pub(super) fn run_worker(shared: &(Mutex<super::worker::SharedState>, Condvar)) {
    let mut cache = DecodedCache::default();
    while let Some(task) = next_task(shared) {
        let wake = match task {
            Task::Slot(target, job) => handle_slot(shared, &mut cache, target, job),
            Task::Inline(job) => handle_inline(shared, &mut cache, job),
        };
        if let Some(wake) = wake {
            wake();
        }
    }
}

fn next_task(shared: &(Mutex<super::worker::SharedState>, Condvar)) -> Option<Task> {
    let guard = shared.0.lock().unwrap_or_else(PoisonError::into_inner);
    let mut state = shared
        .1
        .wait_while(guard, |s| {
            !s.closed && s.picture.is_none() && s.preview.is_none() && s.inline.is_empty()
        })
        .unwrap_or_else(PoisonError::into_inner);
    if state.closed {
        return None;
    }
    if let Some(job) = state.picture.take() {
        return Some(Task::Slot(SlotTarget::Picture, job));
    }
    if let Some(job) = state.preview.take() {
        return Some(Task::Slot(SlotTarget::Preview, job));
    }
    state.inline.pop_front().map(Task::Inline)
}

#[cfg(test)]
pub(super) fn handle_picture(
    shared: &(Mutex<super::worker::SharedState>, Condvar),
    cache: &mut DecodedCache,
    job: PictureJob,
) -> Option<super::ImageWaker> {
    handle_slot(shared, cache, SlotTarget::Picture, job)
}

fn handle_slot(
    shared: &(Mutex<super::worker::SharedState>, Condvar),
    cache: &mut DecodedCache,
    target: SlotTarget,
    job: PictureJob,
) -> Option<super::ImageWaker> {
    let image = cache.get_or_decode(&job.source).and_then(|decoded| {
        let src_size = decoded.dimensions();
        let (max_w, max_h) = match target {
            SlotTarget::Picture => job.limit,
            SlotTarget::Preview => (job.limit.0.min(src_size.0), job.limit.1.min(src_size.1)),
        };
        let (pixels, width, height) = fit_into(&decoded, max_w, max_h);
        (width > 0 && height > 0).then_some(DecodedPicture {
            width,
            height,
            src_size,
            pixels,
        })
    });
    let mut state = shared.0.lock().unwrap_or_else(PoisonError::into_inner);
    if state.closed {
        return None;
    }
    let stale = match target {
        SlotTarget::Picture => state.picture_job != Some(job.job) || state.picture.is_some(),
        SlotTarget::Preview => state.preview_job != Some(job.job) || state.preview.is_some(),
    };
    if stale {
        return None;
    }
    let answer = Some(PictureAnswer {
        job: job.job,
        image,
    });
    match target {
        SlotTarget::Picture => state.picture_answer = answer,
        SlotTarget::Preview => state.preview_answer = answer,
    }
    state.waker.clone()
}

fn handle_inline(
    shared: &(Mutex<super::worker::SharedState>, Condvar),
    cache: &mut DecodedCache,
    job: InlineJob,
) -> Option<super::ImageWaker> {
    let source = ImageSource::Bytes(job.bytes);
    let pixels = cache.get_or_decode(&source).and_then(|decoded| {
        let (pixels, w, h) = fit_into(&decoded, job.limit.0, job.limit.1);
        ((w, h) == (job.key.width, job.key.height)).then_some(pixels)
    });
    let mut state = shared.0.lock().unwrap_or_else(PoisonError::into_inner);
    if state.closed {
        return None;
    }
    state.inline_answers.push(InlineAnswer {
        key: job.key,
        pixels,
    });
    state.waker.clone()
}
