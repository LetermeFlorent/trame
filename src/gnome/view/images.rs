use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use gtk::gdk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::{TermView, hover};
use crate::gnome::worker::{ImageWorker, PreviewKey, WorkerAnswer, WorkerJob, memory_texture};

const MAX_CACHED_PREVIEWS: usize = 64;

#[derive(Clone)]
pub enum InlineTexture {
    Pending,
    Ready(gdk::MemoryTexture),
    Broken,
}

#[derive(Default)]
pub struct PreviewCache {
    entries: HashMap<PreviewKey, InlineTexture>,
    order: VecDeque<PreviewKey>,
}

impl PreviewCache {
    fn insert(&mut self, key: PreviewKey, entry: InlineTexture) {
        if let Some(existing) = self.entries.get_mut(&key) {
            *existing = entry;
            return;
        }
        while self.entries.len() >= MAX_CACHED_PREVIEWS {
            let Some(oldest) = self.order.pop_front() else {
                self.entries.clear();
                break;
            };
            self.entries.remove(&oldest);
        }
        self.order.push_back(key);
        self.entries.insert(key, entry);
    }
}

pub fn ensure_worker(view: &TermView) -> ImageWorker {
    if let Some(worker) = view.imp().worker.borrow().as_ref() {
        return worker.clone();
    }
    let watched = view.downgrade();
    let worker = ImageWorker::spawn(move |answer| {
        let Some(view) = watched.upgrade() else {
            return false;
        };
        apply_answer(&view, answer);
        true
    });
    view.imp().worker.replace(Some(worker.clone()));
    worker
}

pub fn lookup_inline(
    view: &TermView,
    key: PreviewKey,
    limit: (u32, u32),
    bytes: &Arc<[u8]>,
) -> InlineTexture {
    if let Some(found) = view.imp().images.borrow().entries.get(&key) {
        return found.clone();
    }
    let worker = ensure_worker(view);
    view.imp()
        .images
        .borrow_mut()
        .insert(key, InlineTexture::Pending);
    worker.send(WorkerJob::Inline(key, limit, Arc::clone(bytes)));
    InlineTexture::Pending
}

fn apply_answer(view: &TermView, answer: WorkerAnswer) {
    match answer {
        WorkerAnswer::Inline(key, image) => {
            let entry = match image.and_then(memory_texture) {
                Some(texture) => InlineTexture::Ready(texture),
                None => InlineTexture::Broken,
            };
            view.imp().images.borrow_mut().insert(key, entry);
            view.queue_draw();
        }
        WorkerAnswer::Overlay(job, image) => {
            if let Some(ready) = view.imp().overlay_ready.borrow().as_ref() {
                ready(job, image);
            }
        }
        WorkerAnswer::Hover(job, image) => hover::finish_hover(view, job, image),
    }
}
