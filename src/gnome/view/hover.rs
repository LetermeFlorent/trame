use std::path::PathBuf;

use gtk::gdk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use trame::geometry::Rect;
use trame::ui::UiMetrics;
use trame::ui::chrome::hover_preview_limit;

use super::{TermView, images};
use crate::gnome::picture::rules::MAX_PREVIEW_SIDE;
use crate::gnome::worker::{ReadyPixels, WorkerJob, memory_texture};

pub struct ActiveHover {
    pub job: u64,
    pub path: PathBuf,
    pub anchor: Rect,
    pub limit: (u32, u32),
    pub ready: Option<(u32, u32, gdk::MemoryTexture)>,
    pub failed: bool,
}

#[derive(Default)]
pub struct HoverState {
    next_job: u64,
    pub active: Option<ActiveHover>,
}

pub fn set_hover(view: &TermView, path: PathBuf, anchor: Rect) {
    let scale = (view.scale_factor().max(1)) as f32;
    let limit = hover_preview_limit(&UiMetrics::new(scale), MAX_PREVIEW_SIDE);
    let worker = images::ensure_worker(view);
    let mut state = view.imp().hover.borrow_mut();
    if let Some(active) = state.active.as_mut()
        && active.path == path
        && active.limit == limit
    {
        let moved = active.anchor != anchor && active.ready.is_some();
        active.anchor = anchor;
        drop(state);
        if moved {
            view.queue_draw();
        }
        return;
    }
    state.next_job = state.next_job.wrapping_add(1);
    let job = state.next_job;
    state.active = Some(ActiveHover {
        job,
        path: path.clone(),
        anchor,
        limit,
        ready: None,
        failed: false,
    });
    drop(state);
    worker.send(WorkerJob::Hover(job, limit, path));
}

pub fn clear_hover(view: &TermView) {
    let had_ready = view
        .imp()
        .hover
        .borrow_mut()
        .active
        .take()
        .is_some_and(|prev| prev.ready.is_some());
    if had_ready {
        view.queue_draw();
    }
}

pub fn finish_hover(view: &TermView, job: u64, image: Option<ReadyPixels>) {
    let mut state = view.imp().hover.borrow_mut();
    let Some(active) = state.active.as_mut().filter(|item| item.job == job) else {
        return;
    };
    let converted = image.and_then(|ready| {
        let (w, h) = (ready.width, ready.height);
        memory_texture(ready).map(|tex| (w, h, tex))
    });
    match converted {
        Some(ready) => active.ready = Some(ready),
        None => active.failed = true,
    }
    drop(state);
    view.queue_draw();
}
