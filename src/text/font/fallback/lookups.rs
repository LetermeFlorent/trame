use std::collections::{HashSet, VecDeque};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::text::font::discovery::{Finder, FontSource};

pub type Waker = Box<dyn Fn() + Send>;

pub(super) struct Answer {
    pub(super) ch: char,
    pub(super) emoji: bool,
    pub(super) source: Option<FontSource>,
}

struct Worker {
    requests: Sender<(char, bool)>,
    answers: Receiver<Answer>,
}

#[derive(Default)]
pub(super) struct Lookups {
    finder: Option<Finder>,
    waker: Option<Waker>,
    worker: Option<Worker>,
    queued: VecDeque<(char, bool)>,
    asked: HashSet<char>,
    in_flight: bool,
}

impl Lookups {
    pub(super) fn new(finder: Option<Finder>) -> Self {
        Self {
            finder,
            ..Self::default()
        }
    }

    pub(super) fn set_waker(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }

    pub(super) fn can_search(&self) -> bool {
        self.finder.is_some() || self.worker.is_some()
    }

    pub(super) fn is_asked(&self, ch: char) -> bool {
        self.asked.contains(&ch)
    }

    #[cfg(test)]
    pub(super) fn is_busy(&self) -> bool {
        !self.asked.is_empty()
    }

    pub(super) fn ask(&mut self, ch: char, emoji: bool) {
        if self.asked.insert(ch) {
            self.queued.push_back((ch, emoji));
        }
    }

    pub(super) fn next(&mut self) -> Option<(char, bool)> {
        if self.in_flight {
            return None;
        }
        self.queued.pop_front()
    }

    pub(super) fn send(&mut self, ch: char, emoji: bool) -> bool {
        self.in_flight = self
            .worker()
            .is_some_and(|worker| worker.requests.send((ch, emoji)).is_ok());
        self.in_flight
    }

    pub(super) fn answer(&mut self) -> Option<Answer> {
        let answer = self.worker.as_ref()?.answers.try_recv().ok()?;
        self.in_flight = false;
        Some(answer)
    }

    pub(super) fn settle(&mut self, ch: char) {
        self.asked.remove(&ch);
    }

    fn worker(&mut self) -> Option<&Worker> {
        if self.worker.is_none() {
            let finder = self.finder.take()?;
            self.worker = spawn(finder, self.waker.take());
        }
        self.worker.as_ref()
    }
}

fn spawn(mut finder: Finder, waker: Option<Waker>) -> Option<Worker> {
    let (requests, inbox) = mpsc::channel::<(char, bool)>();
    let (outbox, answers) = mpsc::channel();
    let work = move || {
        for (ch, emoji) in inbox {
            let source = finder(ch, emoji);
            if outbox.send(Answer { ch, emoji, source }).is_err() {
                break;
            }
            if let Some(wake) = &waker {
                wake();
            }
        }
    };
    thread::Builder::new()
        .name("font-lookup".to_owned())
        .spawn(work)
        .ok()?;
    Some(Worker { requests, answers })
}
