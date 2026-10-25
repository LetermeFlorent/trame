use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use super::threads::spawn;
use super::{Notifier, SessionEvent, SessionId};

const SYNC_LIMIT: Duration = Duration::from_secs(1);

pub struct SyncHold {
    id: SessionId,
    pending: Arc<AtomicBool>,
    notify: Notifier,
    started: Option<Instant>,
    generation: u64,
    current: Arc<AtomicU64>,
    wake: Option<Sender<(u64, Instant)>>,
}

impl SyncHold {
    pub fn new(id: SessionId, pending: Arc<AtomicBool>, notify: Notifier) -> Self {
        Self {
            id,
            pending,
            notify,
            started: None,
            generation: 0,
            current: Arc::new(AtomicU64::new(0)),
            wake: None,
        }
    }

    pub fn holds(&mut self, synchronized: bool) -> bool {
        if !synchronized {
            if self.started.take().is_some() {
                self.current.store(0, Ordering::Release);
            }
            return false;
        }
        let started = match self.started {
            Some(started) => started,
            None => self.start(),
        };
        started.elapsed() < SYNC_LIMIT
    }

    fn start(&mut self) -> Instant {
        let now = Instant::now();
        self.started = Some(now);
        self.generation += 1;
        self.current.store(self.generation, Ordering::Release);
        let alarm = (self.generation, now + SYNC_LIMIT);
        let sent = self.waker().is_some_and(|wake| wake.send(alarm).is_ok());
        if !sent {
            self.wake = None;
        }
        now
    }

    fn waker(&mut self) -> Option<&Sender<(u64, Instant)>> {
        if self.wake.is_none() {
            let (sender, queue) = mpsc::channel::<(u64, Instant)>();
            let (id, current) = (self.id, self.current.clone());
            let (pending, notify) = (self.pending.clone(), self.notify.clone());
            let spawned = spawn("trame-sync", move || {
                for (generation, deadline) in queue {
                    thread::sleep(deadline.saturating_duration_since(Instant::now()));
                    let still_held = current.load(Ordering::Acquire) == generation;
                    if still_held && !pending.swap(true, Ordering::AcqRel) {
                        notify(SessionEvent::Output(id));
                    }
                }
            });
            self.wake = spawned.ok().map(|()| sender);
        }
        self.wake.as_ref()
    }
}
