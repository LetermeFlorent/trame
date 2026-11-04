use trame::preferences::Preferences;

use super::{MANY, host, hosts};

#[test]
fn a_used_connection_moves_first_without_duplicates_and_nothing_is_dropped() {
    let base = Preferences {
        ssh_hosts: hosts(MANY),
        ..Preferences::default()
    };
    let again = base.remember_host(host("user@host3", None));
    assert_eq!(again.ssh_hosts.len(), MANY);
    assert_eq!(again.ssh_hosts[0], host("user@host3", None));
    assert_eq!(again.ssh_hosts[4], host("user@host4", None));
    let fresh = base.remember_host(host("new@box", Some(2200)));
    assert_eq!(fresh.ssh_hosts.len(), MANY + 1);
    assert_eq!(fresh.ssh_hosts[0], host("new@box", Some(2200)));
    assert_eq!(fresh.ssh_hosts[MANY], hosts(MANY)[MANY - 1]);
}

#[test]
fn a_changed_connection_keeps_its_place_and_merges_a_duplicate() {
    let known = hosts(4);
    let base = Preferences {
        ssh_hosts: known.clone(),
        ..Preferences::default()
    };
    let renamed = base.replace_host(&known[2], host("me@other", Some(2200)));
    assert_eq!(renamed.ssh_hosts.len(), 4);
    assert_eq!(renamed.ssh_hosts[2], host("me@other", Some(2200)));
    let merged = base.replace_host(&known[2], known[0].clone());
    assert_eq!(
        merged.ssh_hosts,
        [known[1].clone(), known[0].clone(), known[3].clone()]
    );
    let unknown = base.replace_host(&host("gone", None), host("fresh", None));
    assert_eq!(unknown.ssh_hosts[0], host("fresh", None));
    assert_eq!(unknown.ssh_hosts.len(), 5);
}

#[test]
fn forgetting_a_connection_removes_only_that_one() {
    let known = hosts(3);
    let base = Preferences {
        ssh_hosts: known.clone(),
        ..Preferences::default()
    };
    let forgotten = base.forget_host(&known[1]);
    assert_eq!(forgotten.ssh_hosts, [known[0].clone(), known[2].clone()]);
    assert_eq!(base.forget_host(&host("gone", None)), base);
}
