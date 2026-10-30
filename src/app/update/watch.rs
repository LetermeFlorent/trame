use std::thread;
use std::time::Duration;

use super::version::Version;
use super::{UpdateStatus, download};

const CHECK_EVERY: Duration = Duration::from_secs(60 * 60);

pub fn newer_release() -> Option<Version> {
    let latest = download::latest()?;
    let newer = Version::current().is_some_and(|current| latest > current);
    newer.then_some(latest)
}

pub fn watch(notify: &dyn Fn(UpdateStatus)) {
    let wait = || {
        thread::sleep(CHECK_EVERY);
        true
    };
    follow(newer_release, wait, notify);
}

fn follow(
    mut check: impl FnMut() -> Option<Version>,
    mut wait: impl FnMut() -> bool,
    notify: &dyn Fn(UpdateStatus),
) {
    let mut offered = None;
    loop {
        let latest = check();
        if let Some(status) = offer(latest, offered) {
            notify(status);
        }
        offered = latest.or(offered);
        if !wait() {
            return;
        }
    }
}

fn offer(latest: Option<Version>, offered: Option<Version>) -> Option<UpdateStatus> {
    latest
        .filter(|latest| Some(*latest) != offered)
        .map(|latest| UpdateStatus::Available(latest.number()))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    fn version(text: &str) -> Version {
        Version::parse(text).expect(text)
    }

    fn watched(checks: &[Option<&str>]) -> Vec<UpdateStatus> {
        let mut answers = checks.iter().map(|check| check.map(version));
        let mut rounds = checks.len();
        let sent = RefCell::new(Vec::new());
        let wait = || {
            rounds -= 1;
            rounds > 0
        };
        follow(|| answers.next().flatten(), wait, &|status| {
            sent.borrow_mut().push(status)
        });
        sent.into_inner()
    }

    #[test]
    fn the_watch_only_ever_offers_a_version() {
        let sent = watched(&[None, Some("9.0.0"), Some("9.0.0"), None, Some("9.1.0")]);
        let expected = ["9.0.0", "9.1.0"].map(|number| UpdateStatus::Available(number.into()));
        assert_eq!(sent, expected);
    }

    #[test]
    fn the_same_version_is_offered_once() {
        assert_eq!(offer(Some(version("9.0.0")), Some(version("9.0.0"))), None);
        assert_eq!(offer(None, Some(version("9.0.0"))), None);
    }

    #[test]
    fn the_first_answer_comes_without_waiting() {
        let sent = watched(&[Some("9.0.0")]);
        assert_eq!(sent, [UpdateStatus::Available("9.0.0".into())]);
    }
}
