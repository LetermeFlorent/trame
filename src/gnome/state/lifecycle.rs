use std::time::{Duration, Instant};

use trame::desk::Closing;
use trame::input::Command;
use trame::session::{SessionEvent, SessionId};
use trame::workspace::{PaneLayout, Snapshot, WindowSnapshot};

use super::WindowState;

const WORK_FRAME: Duration = Duration::from_millis(33);

impl WindowState {
    pub fn ask_close(&mut self, closing: Closing) {
        let effects = self.desk.ask_close(closing, self.loading);
        self.apply(effects);
    }

    pub fn run_command(&mut self, exiting: bool, command: Command) {
        let panes = PaneLayout {
            panes: self.frame.panes.iter().map(|p| (p.id, p.rect)).collect(),
            dividers: Vec::new(),
        };
        if let Some(effects) = self.desk.command(exiting, command, &panes) {
            self.apply(effects);
        }
    }

    pub fn session_event(&mut self, exiting: bool, event: SessionEvent) -> Option<SessionId> {
        match event {
            SessionEvent::Output(id) => {
                self.session_output(id);
                Some(id)
            }
            SessionEvent::Exited(id) => {
                let effects = self.desk.shell_exited(exiting, id);
                self.apply(effects);
                None
            }
        }
    }

    fn session_output(&mut self, id: SessionId) {
        self.desk.power.output(id, self.desk.sessions.resting(id));
        self.desk.sessions.output(id);
        if self.desk.sessions.notice_exit(id) {
            self.relayout();
        }
        let shown = self.frame.panes.iter().any(|p| p.active == Some(id));
        let relabeled = self
            .desk
            .sessions
            .get(id)
            .zip(self.labels.get(&id))
            .is_some_and(|(s, old)| *old != s.label());
        if shown || relabeled {
            self.dirty = true;
        }
    }

    pub fn snapshot(&self) -> Option<WindowSnapshot> {
        let w = (self.window_size.0 / self.window_scale).round() as u32;
        let h = (self.window_size.1 / self.window_scale).round() as u32;
        (!self.hidden && w > 0 && h > 0).then_some(WindowSnapshot {
            width: w,
            height: h,
            maximized: self.maximized,
        })
    }

    pub fn restore_session(&mut self, saved: Option<&Snapshot>) -> bool {
        if !self.desk.restore(saved) {
            return false;
        }
        self.relayout();
        let g = self.grids();
        self.desk.start_shown(&g);
        if !self.desk.settle_restore() {
            return false;
        }
        self.changed();
        true
    }

    pub fn about_to_wait(&mut self, now: Instant) -> Option<Instant> {
        self.desk.power.set_background(self.hidden);
        self.desk.power.tick(now);
        for id in self.desk.power.take_rested() {
            if let Some(s) = self.desk.sessions.get(id) {
                s.terminal().compact();
            }
        }
        let snap = self.snapshot();
        let save = self
            .desk
            .save_deadline(now + Duration::from_millis(10), || snap);
        self.apply(save.effects);
        let text = self.desk.transcript_deadline(now);
        let secret = self.desk.sessions.secret_deadline(now);
        let remote = self.desk.redial(now);
        self.apply(remote.effects);
        let work = self.work_deadline(now);
        save.value
            .into_iter()
            .chain(text)
            .chain(secret)
            .chain(remote.value)
            .chain(work)
            .chain(self.desk.power.deadline())
            .min()
    }

    fn work_deadline(&mut self, now: Instant) -> Option<Instant> {
        if !self.frame.shows_work() || self.hidden {
            return None;
        }
        let next = self.motion.now() + WORK_FRAME;
        if next <= now {
            self.dirty = true;
            Some(now + WORK_FRAME)
        } else {
            Some(next)
        }
    }
}
