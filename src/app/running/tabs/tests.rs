use std::time::{Duration, Instant};

use trame::session::SessionId;

use super::restarts::Restarts;

const SOON: Duration = Duration::from_secs(1);
const LATER: Duration = Duration::from_secs(60);

#[test]
fn a_shell_that_exits_twice_at_once_is_not_restarted() {
    let mut restarts = Restarts::default();
    let launch = Instant::now();
    assert!(restarts.allows(SessionId(1), launch + SOON));
    restarts.opened(SessionId(2), launch + SOON);
    assert!(!restarts.allows(SessionId(2), launch + SOON + SOON));
}

#[test]
fn a_shell_that_ran_for_a_while_is_restarted() {
    let mut restarts = Restarts::default();
    let launch = Instant::now();
    restarts.opened(SessionId(2), launch);
    assert!(restarts.allows(SessionId(2), launch + LATER));
    assert!(restarts.allows(SessionId(3), launch + SOON));
}
