use std::net::{TcpStream, ToSocketAddrs};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use trame::session::SessionId;

use super::Sessions;

const SSH_FAILED: i32 = 255;
const SETTLED: Duration = Duration::from_secs(10);
pub(super) const RETRY: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(500);
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) struct Remote {
    address: String,
    port: u16,
    link: Link,
}

enum Link {
    Up {
        since: Instant,
        proven: bool,
    },
    Down {
        probe: Option<JoinHandle<bool>>,
        next: Instant,
    },
}

impl Remote {
    fn is_down(&self) -> bool {
        matches!(self.link, Link::Down { .. })
    }
}

impl Sessions {
    pub fn watch_remote(&mut self, id: SessionId, address: String, port: u16) {
        let proven = self.remotes.contains_key(&id);
        let link = Link::Up {
            since: Instant::now(),
            proven,
        };
        let remote = Remote {
            address,
            port,
            link,
        };
        self.remotes.insert(id, remote);
    }

    pub fn forget_remote(&mut self, id: SessionId) {
        self.remotes.remove(&id);
    }

    pub fn offline(&self, id: SessionId) -> bool {
        self.remotes.get(&id).is_some_and(Remote::is_down)
    }

    pub fn notice_exit(&mut self, id: SessionId) -> bool {
        let session = self.map.get(&id);
        let code = session.and_then(|session| session.terminal().screen_mut().take_exit());
        code.is_some_and(|code| self.ended(id, code))
    }

    pub(super) fn ended(&mut self, id: SessionId, code: i32) -> bool {
        let Some(remote) = self.remotes.get_mut(&id) else {
            return false;
        };
        let Link::Up { since, proven } = remote.link else {
            return false;
        };
        if code == SSH_FAILED && (proven || since.elapsed() >= SETTLED) {
            let next = Instant::now() + RETRY;
            remote.link = Link::Down { probe: None, next };
            return true;
        }
        self.remotes.remove(&id);
        false
    }

    pub fn redial_due(&mut self, now: Instant) -> Vec<SessionId> {
        let mut due = Vec::new();
        for (id, remote) in &mut self.remotes {
            let Remote {
                address,
                port,
                link: Link::Down { probe, next },
            } = remote
            else {
                continue;
            };
            match probe.take() {
                Some(handle) if !handle.is_finished() => *probe = Some(handle),
                Some(handle) => match handle.join().unwrap_or(false) {
                    true => due.push(*id),
                    false => *next = now + RETRY,
                },
                None if now >= *next => *probe = spawn_probe(address.clone(), *port),
                None => {}
            }
        }
        due
    }

    pub fn remote_deadline(&self, now: Instant) -> Option<Instant> {
        let down = self.remotes.values().any(Remote::is_down);
        down.then_some(now + POLL)
    }
}

fn spawn_probe(address: String, port: u16) -> Option<JoinHandle<bool>> {
    thread::Builder::new()
        .name("ssh-probe".into())
        .spawn(move || reachable(&address, port))
        .ok()
}

fn reachable(address: &str, port: u16) -> bool {
    (address, port)
        .to_socket_addrs()
        .is_ok_and(|mut addresses| {
            addresses.any(|address| TcpStream::connect_timeout(&address, PROBE_TIMEOUT).is_ok())
        })
}
