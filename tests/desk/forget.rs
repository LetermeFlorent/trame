use std::process;

use trame::desk::{Closing, Confirm, Effect};
use trame::locale::ALIAS_SLOT;
use trame::preferences::SshHost;

use crate::tabs::opened;

fn host(target: String) -> SshHost {
    SshHost { target, port: None }
}

fn unknown_alias() -> SshHost {
    host(format!("trame-test-forget-{}", process::id()))
}

#[test]
fn a_host_not_written_by_hand_goes_at_once() {
    let (mut desk, _) = opened();
    let known = host("me@box.invalid".to_owned());
    desk.preferences = desk.preferences.remember_host(known.clone());
    let effects = desk.ask_forget(known.clone());
    assert_eq!(effects.as_slice(), [Effect::Changed, Effect::HostsChanged]);
    assert!(desk.closing.is_none());
    assert!(!desk.preferences.ssh_hosts.contains(&known));
}

#[test]
fn a_pending_forget_asks_to_remove_the_alias() {
    let (mut desk, _) = opened();
    let known = unknown_alias();
    desk.closing = Some(Closing::Forget(known.clone()));
    let question = desk.question().expect("a question");
    assert_eq!(question.confirm, Confirm::Remove);
    let alias = known.alias().expect("an alias");
    let title = desk.strings.remove_question.replace(ALIAS_SLOT, alias);
    assert_eq!(question.title, title);
}

#[test]
fn a_confirmed_forget_drops_the_host() {
    let (mut desk, _) = opened();
    let known = unknown_alias();
    desk.preferences = desk.preferences.remember_host(known.clone());
    desk.closing = Some(Closing::Forget(known.clone()));
    let effects = desk.settle_closing(true);
    let dropped = [Effect::Changed, Effect::HostsChanged, Effect::Changed];
    assert_eq!(effects.as_slice(), dropped);
    assert!(desk.closing.is_none());
    assert!(!desk.preferences.ssh_hosts.contains(&known));
}

#[test]
fn a_closing_without_a_question_is_dropped() {
    let (mut desk, _) = opened();
    desk.closing = Some(Closing::Forget(host("me@box.invalid".to_owned())));
    assert!(desk.pending_question().is_none());
    assert!(desk.closing.is_none());
}
