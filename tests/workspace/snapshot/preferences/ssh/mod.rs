mod config;
mod edit;
mod form;
mod list;
mod prompt;
mod vault;

use trame::preferences::{Preferences, SshHost};

pub const MANY: usize = 20;

pub fn host(target: &str, port: Option<u16>) -> SshHost {
    SshHost {
        target: target.to_owned(),
        port,
    }
}

pub fn hosts(count: usize) -> Vec<SshHost> {
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
fn only_a_bare_name_can_be_a_config_alias() {
    assert_eq!(host("prod", None).alias(), Some("prod"));
    assert_eq!(host("me@prod", None).alias(), None);
    assert_eq!(host("prod", Some(2200)).alias(), None);
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
        ssh_hosts: hosts(MANY),
        ..Preferences::default()
    };
    assert_eq!(many.sanitized().ssh_hosts, hosts(MANY));
}

#[test]
fn preferences_without_connections_write_no_list() {
    let json = serde_json::to_string(&Preferences::default()).expect("serializes");
    assert!(!json.contains("ssh_hosts"));
}
