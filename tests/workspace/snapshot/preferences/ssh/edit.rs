use std::fs;

use trame::preferences::{HostEntry, SshConfig, entry_in, with_host, without_host};

use super::host;

const THREE: &str = "Host a\n    HostName 1.1.1.1\n\n\
Host b\n    HostName b\n    User me\n\n\
Host c\n    HostName c\n";

fn entry(address: &str, user: Option<&str>, port: Option<u16>) -> HostEntry {
    HostEntry {
        address: address.to_owned(),
        user: user.map(str::to_owned),
        port,
    }
}

#[test]
fn removing_a_host_drops_its_block_and_one_blank_line() {
    assert_eq!(
        without_host(THREE, "b"),
        "Host a\n    HostName 1.1.1.1\n\nHost c\n    HostName c\n"
    );
    assert_eq!(
        without_host(THREE, "c"),
        "Host a\n    HostName 1.1.1.1\n\nHost b\n    HostName b\n    User me\n"
    );
    assert!(without_host(THREE, "a").starts_with("Host b\n"));
    assert_eq!(without_host(THREE, "missing"), THREE);
}

#[test]
fn a_host_sharing_its_line_only_loses_its_name() {
    let shared = "Host one two three\n    User x\n";
    assert_eq!(without_host(shared, "two"), "Host one three\n    User x\n");
    assert_eq!(without_host(shared, "one"), "Host two three\n    User x\n");
    assert_eq!(without_host("Host \"two\" one\n", "two"), "Host one\n");
}

#[test]
fn every_copy_goes_and_match_blocks_stay() {
    let twice = "Host x\n  User a\nHost x\n  User b\nHost y\n";
    assert_eq!(without_host(twice, "x"), "Host y\n");
    let matched = "Host x\n  User a\nMatch host x\n  User root\n";
    assert_eq!(without_host(matched, "x"), "Match host x\n  User root\n");
}

#[test]
fn windows_files_keep_their_endings_and_mark() {
    let windows = "\u{feff}Host a\r\n  User u\r\n\r\nHost b\r\n  User v\r\n";
    assert_eq!(without_host(windows, "a"), "\u{feff}Host b\r\n  User v\r\n");
    let edited = with_host("Host a\r\n  HostName a\r\n", "a", &host("b", None));
    assert_eq!(
        edited,
        Some(("Host b\r\n  HostName b\r\n".to_owned(), "b".to_owned()))
    );
}

#[test]
fn an_entry_reads_the_first_value_of_each_setting() {
    let text = "Host prod\n  hostname 1.2.3.4\n  USER me\n  Port 22\n  HostName no\n\
Host x\n  Port=2200\nHost bare\n";
    assert_eq!(
        entry_in(text, "prod"),
        Some(entry("1.2.3.4", Some("me"), None))
    );
    assert_eq!(entry_in(text, "x"), Some(entry("x", None, Some(2200))));
    assert_eq!(entry_in(text, "bare"), Some(entry("bare", None, None)));
    assert_eq!(entry_in(text, "missing"), None);
}

#[test]
fn a_host_named_after_its_address_follows_the_new_address() {
    let text = "Host 10.0.0.2\n    HostName 10.0.0.2\n    User root\n    Port 2222\n";
    let edited = with_host(text, "10.0.0.2", &host("admin@10.0.0.9", None));
    assert_eq!(
        edited,
        Some((
            "Host 10.0.0.9\n    HostName 10.0.0.9\n    User admin\n".to_owned(),
            "10.0.0.9".to_owned()
        ))
    );
    assert_eq!(with_host(text, "missing", &host("x", None)), None);
}

#[test]
fn a_chosen_name_and_other_settings_are_kept() {
    let text = "Host prod # main\n\tHostName 1.2.3.4\n\tIdentityFile ~/.ssh/id\n\tUser old\n\n\
Host other\n";
    let (edited, alias) =
        with_host(text, "prod", &host("new@5.6.7.8", Some(2200))).expect("the block exists");
    assert_eq!(alias, "prod");
    assert_eq!(
        edited,
        "Host prod # main\n\tHostName 5.6.7.8\n\tUser new\n\tPort 2200\n\
\tIdentityFile ~/.ssh/id\n\nHost other\n"
    );
    let taken = "Host a\n  HostName a\nHost b\n  HostName 9.9.9.9\n";
    let (edited, alias) = with_host(taken, "a", &host("b", None)).expect("the block exists");
    assert_eq!(alias, "a");
    assert_eq!(edited, "Host a\n  HostName b\nHost b\n  HostName 9.9.9.9\n");
}

#[test]
fn the_file_loses_or_rewrites_only_the_chosen_block() {
    let folder = std::env::temp_dir().join(format!("trame-ssh-edit-{}", std::process::id()));
    let path = folder.join("config");
    fs::create_dir_all(&folder).expect("the test folder is made");
    fs::write(&path, "Host a\n  HostName a\n\nHost b\n  HostName b\n").expect("written");
    let config = SshConfig::at(&path);
    assert_eq!(
        config.entry("b").expect("read"),
        Some(entry("b", None, None))
    );
    let kept = config
        .replace("a", &host("me@c", Some(2200)))
        .expect("rewritten");
    assert_eq!(kept, host("c", None));
    assert_eq!(config.aliases().expect("read"), ["c", "b"]);
    config.remove("b").expect("removed");
    let text = fs::read_to_string(&path).expect("read");
    assert_eq!(text, "Host c\n  HostName c\n  User me\n  Port 2200\n");
    let added = config.replace("gone", &host("d", None)).expect("appended");
    assert_eq!(added, host("d", None));
    assert!(
        fs::read_to_string(&path)
            .expect("read")
            .ends_with("\nHost d\n    HostName d\n")
    );
    fs::remove_dir_all(&folder).expect("the test folder is removed");
}
