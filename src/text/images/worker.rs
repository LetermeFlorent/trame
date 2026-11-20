use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread;

use super::decode::run_worker;
use super::source::{InlineAnswer, InlineJob, PictureAnswer, PictureJob, SlotTarget};

pub type ImageWaker = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
pub(super) struct SharedState {
    pub(super) picture_job: Option<u64>,
    pub(super) picture: Option<PictureJob>,
    pub(super) picture_answer: Option<PictureAnswer>,
    pub(super) preview_job: Option<u64>,
    pub(super) preview: Option<PictureJob>,
    pub(super) preview_answer: Option<PictureAnswer>,
    pub(super) inline: VecDeque<InlineJob>,
    pub(super) inline_answers: Vec<InlineAnswer>,
    pub(super) waker: Option<ImageWaker>,
    pub(super) closed: bool,
}

pub(super) struct ImageWorker {
    pub(super) shared: Arc<(Mutex<SharedState>, Condvar)>,
    spawned: bool,
}

impl Default for ImageWorker {
    fn default() -> Self {
        Self {
            shared: Arc::new((Mutex::new(SharedState::default()), Condvar::new())),
            spawned: false,
        }
    }
}

impl Drop for ImageWorker {
    fn drop(&mut self) {
        let mut state = self.shared.0.lock().unwrap_or_else(PoisonError::into_inner);
        state.closed = true;
        self.shared.1.notify_all();
    }
}

impl ImageWorker {
    pub(super) fn set_waker(&mut self, waker: ImageWaker) {
        let mut state = self.shared.0.lock().unwrap_or_else(PoisonError::into_inner);
        state.waker = Some(waker);
    }

    pub(super) fn push_inline(&mut self, job: InlineJob) {
        self.ensure_spawned();
        let mut state = self.shared.0.lock().unwrap_or_else(PoisonError::into_inner);
        state.inline.push_back(job);
        self.shared.1.notify_one();
    }

    pub(super) fn set_slot(&mut self, target: SlotTarget, job: PictureJob) {
        self.ensure_spawned();
        let mut state = self.shared.0.lock().unwrap_or_else(PoisonError::into_inner);
        match target {
            SlotTarget::Picture => {
                state.picture_job = Some(job.job);
                state.picture = Some(job);
                state.picture_answer = None;
            }
            SlotTarget::Preview => {
                state.preview_job = Some(job.job);
                state.preview = Some(job);
                state.preview_answer = None;
            }
        }
        self.shared.1.notify_one();
    }

    pub(super) fn cancel_slot(&mut self, target: SlotTarget) {
        let mut state = self.shared.0.lock().unwrap_or_else(PoisonError::into_inner);
        match target {
            SlotTarget::Picture => {
                state.picture_job = None;
                state.picture = None;
                state.picture_answer = None;
            }
            SlotTarget::Preview => {
                state.preview_job = None;
                state.preview = None;
                state.preview_answer = None;
            }
        }
    }

    pub(super) fn has_slot_job(&self, target: SlotTarget, job: u64) -> bool {
        let state = self.shared.0.lock().unwrap_or_else(PoisonError::into_inner);
        match target {
            SlotTarget::Picture => state.picture_job == Some(job),
            SlotTarget::Preview => state.preview_job == Some(job),
        }
    }

    pub(super) fn drain(
        &self,
    ) -> (
        Vec<InlineAnswer>,
        Option<PictureAnswer>,
        Option<PictureAnswer>,
    ) {
        let mut state = self.shared.0.lock().unwrap_or_else(PoisonError::into_inner);
        (
            std::mem::take(&mut state.inline_answers),
            state.picture_answer.take(),
            state.preview_answer.take(),
        )
    }

    fn ensure_spawned(&mut self) {
        if self.spawned {
            return;
        }
        let shared = Arc::clone(&self.shared);
        self.spawned = thread::Builder::new()
            .name("image-decode".to_owned())
            .spawn(move || run_worker(&shared))
            .is_ok();
    }
}
