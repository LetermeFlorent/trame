use std::fmt::Display;

use super::session::{FrameOutcome, Session};
use trame::render::{Atlas, ReadbackError};

const MAX_HARDWARE_LOSSES: u8 = 2;

#[derive(Default)]
pub(super) struct RecoveryPolicy {
    pub(super) prefer_software: bool,
    pub(super) hardware_losses: u8,
}

pub(super) struct GraphicsState<S> {
    pub(super) active: Option<Session<S>>,
    pub(super) policy: RecoveryPolicy,
    pub(super) fresh: bool,
    pub(super) failed: bool,
    pub(super) redraw: bool,
}

impl<S> GraphicsState<S> {
    pub(super) fn new(session: Session<S>) -> Self {
        let policy = RecoveryPolicy {
            prefer_software: session.software,
            hardware_losses: 0,
        };
        Self {
            policy,
            active: Some(session),
            fresh: true,
            failed: false,
            redraw: false,
        }
    }

    pub(super) fn needs_recovery(&self) -> bool {
        !self.failed && self.active.as_ref().is_none_or(Session::needs_recovery)
    }

    pub(super) fn render_image_with<E: Display>(
        &mut self,
        atlas: &mut Atlas,
        render: impl FnOnce(&mut Session<S>, &mut Atlas) -> Result<Vec<u8>, ReadbackError>,
        open: impl FnOnce(&mut Atlas, bool) -> Result<Session<S>, E>,
        log: &mut dyn FnMut(&str),
    ) -> Result<Vec<u8>, ReadbackError> {
        if self.failed {
            return Err(ReadbackError::Lost);
        }
        if self.needs_recovery() {
            self.recover(atlas, open, log);
            return Err(ReadbackError::Lost);
        }
        let result = match &mut self.active {
            Some(active) => render(active, atlas),
            None => return Err(ReadbackError::Lost),
        };
        let lost = self.needs_recovery();
        if result.is_err() || lost {
            self.recover(atlas, open, log);
        }
        if lost {
            Err(ReadbackError::Lost)
        } else {
            result
        }
    }

    pub(super) fn present_with<E: Display>(
        &mut self,
        atlas: &mut Atlas,
        draw: impl FnOnce(&mut Session<S>, &mut Atlas) -> FrameOutcome,
        open: impl FnOnce(&mut Atlas, bool) -> Result<Session<S>, E>,
        log: &mut dyn FnMut(&str),
    ) -> bool {
        if self.failed {
            return false;
        }
        if self.needs_recovery() {
            self.recover(atlas, open, log);
            return false;
        }
        let outcome = match &mut self.active {
            Some(active) => draw(active, atlas),
            None => return false,
        };
        match outcome {
            FrameOutcome::Presented => {
                self.policy.hardware_losses = 0;
                true
            }
            FrameOutcome::Skipped => false,
            FrameOutcome::Lost => {
                self.recover(atlas, open, log);
                false
            }
        }
    }

    fn recover<E: Display>(
        &mut self,
        atlas: &mut Atlas,
        open: impl FnOnce(&mut Atlas, bool) -> Result<Session<S>, E>,
        log: &mut dyn FnMut(&str),
    ) -> bool {
        if self.failed {
            return false;
        }
        let ok = recover_session(&mut self.active, &mut self.policy, atlas, open, log);
        self.failed = !ok;
        if ok {
            self.fresh = true;
            self.redraw = true;
        }
        ok
    }
}

pub(super) fn recover_session<S, E: Display>(
    session: &mut Option<Session<S>>,
    policy: &mut RecoveryPolicy,
    atlas: &mut Atlas,
    open: impl FnOnce(&mut Atlas, bool) -> Result<Session<S>, E>,
    log: &mut dyn FnMut(&str),
) -> bool {
    if session.as_ref().is_some_and(|active| active.software) {
        policy.prefer_software = true;
    } else if !policy.prefer_software {
        policy.hardware_losses = policy.hardware_losses.saturating_add(1);
        if policy.hardware_losses >= MAX_HARDWARE_LOSSES {
            policy.prefer_software = true;
            log("repeated graphics device losses, switching to the software adapter");
        }
    }
    *session = None;
    match open(atlas, policy.prefer_software) {
        Ok(fresh) => {
            policy.prefer_software |= fresh.software;
            atlas.invalidate();
            *session = Some(fresh);
            log("the graphics device recovered");
            true
        }
        Err(error) => {
            log(&format!("the graphics device could not recover: {error}"));
            false
        }
    }
}
