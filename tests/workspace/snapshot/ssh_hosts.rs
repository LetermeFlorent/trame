use trame::preferences::{MAX_SSH_HOSTS, Preferences, SshHost};

fn host(target: &str, port: Option<u16>) -> SshHost {
    SshHost {
        target: target.to_owned(),
        port,
    }
}

fn hosts(count: usize) -> Vec<SshHost> {
    (0..count)
        .map(|index| host(&format!("user@host{index}"), None))
        .collect()
}

#[test]
fn a_typed_connection_reads_its_user_host_and_port() {
    let cases = [
        ("user@example.com", host("user@example.com", None)),
        ("  server-1.lan  ", host("server-1.lan", None)),
        ("me@10.0.0.2:2222", host("me@10.0.0.2", Some(2222))),
        ("me@box:22", host("me@box", None)),
        ("my_user@my-box", host("my_user@my-box", None)),
    ];
    for (typed, expected) in cases {
        assert_eq!(SshHost::parse(typed), Some(expected), "{typed}");
    }
}

#[test]
fn a_typed_connection_that_could_run_something_else_is_refused() {
    let refused = [
        "",
        "@host",
        "user@",
        "a@b@c",
        "host:",
        "host:0",
        "host:70000",
        "-oProxyCommand=calc",
        "user@-host",
        "host name",
        "host;calc",
        "host&calc",
        "host|calc",
        "$(calc)",
        "host`calc`",
        "hôte",
    ];
    for typed in refused {
        assert_eq!(SshHost::parse(typed), None, "{typed}");
    }
}

#[test]
fn the_command_names_the_port_only_when_it_is_not_the_usual_one() {
    assert_eq!(host("me@box", None).command(), "ssh me@box\r");
    assert_eq!(host("me@box", Some(2222)).command(), "ssh -p 2222 me@box\r");
    assert_eq!(host("me@box", Some(2222)).label(), "me@box:2222");
    assert_eq!(host("me@box", None).label(), "me@box");
}

#[test]
fn a_used_connection_moves_first_without_duplicates_and_the_list_stays_short() {
    let base = Preferences {
        ssh_hosts: hosts(MAX_SSH_HOSTS),
        ..Preferences::default()
    };
    let again = base.remember_host(host("user@host3", None));
    assert_eq!(again.ssh_hosts.len(), MAX_SSH_HOSTS);
    assert_eq!(again.ssh_hosts[0], host("user@host3", None));
    assert_eq!(again.ssh_hosts[4], host("user@host4", None));
    let fresh = base.remember_host(host("new@box", Some(2200)));
    assert_eq!(fresh.ssh_hosts.len(), MAX_SSH_HOSTS);
    assert_eq!(fresh.ssh_hosts[0], host("new@box", Some(2200)));
    assert!(
        !fresh
            .ssh_hosts
            .contains(&hosts(MAX_SSH_HOSTS)[MAX_SSH_HOSTS - 1])
    );
}

#[test]
fn forgetting_a_connection_removes_only_that_one() {
    let base = Preferences {
        ssh_hosts: hosts(3),
        ..Preferences::default()
    };
    let forgotten = base.forget_host(1);
    assert_eq!(
        forgotten.ssh_hosts,
        [hosts(3)[0].clone(), hosts(3)[2].clone()]
    );
    assert_eq!(base.forget_host(3), base);
}

#[test]
fn saved_connections_survive_json_and_bad_ones_are_dropped_on_load() {
    let base = Preferences {
        ssh_hosts: vec![host("me@box", Some(2222)), host("server", None)],
        ..Preferences::default()
    };
    let json = serde_json::to_string(&base).expect("serializes");
    let back: Preferences = serde_json::from_str(&json).expect("parses");
    assert_eq!(back.sanitized(), base);
    let saved = r#"{"font_size":14.0,"ssh_hosts":[
        {"target":"me@box","port":22},
        {"target":"me@box"},
        {"target":"box;calc"},
        {"target":"other","port":0}
    ]}"#;
    let loaded: Preferences = serde_json::from_str(saved).expect("parses");
    assert_eq!(loaded.sanitized().ssh_hosts, [host("me@box", None)]);
    let many = Preferences {
        ssh_hosts: hosts(MAX_SSH_HOSTS + 3),
        ..Preferences::default()
    };
    assert_eq!(many.sanitized().ssh_hosts, hosts(MAX_SSH_HOSTS));
}

#[test]
fn preferences_without_connections_write_no_list() {
    let json = serde_json::to_string(&Preferences::default()).expect("serializes");
    assert!(!json.contains("ssh_hosts"));
}
