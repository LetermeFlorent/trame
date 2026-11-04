use std::fs;

use trame::preferences::{SshConfig, without_host, written_by_hand};

#[test]
fn a_block_with_a_pattern_is_never_removed() {
    let text = "Host work *.corp\n    User me\n";
    assert_eq!(without_host(text, "work"), "Host *.corp\n    User me\n");
}

#[test]
fn removing_one_alias_keeps_the_others() {
    let text = "Host work work+old\n    HostName w.example\n";
    let kept = "Host work+old\n    HostName w.example\n";
    assert_eq!(without_host(text, "work"), kept);
}

#[test]
fn only_blocks_trame_wrote_go_without_asking() {
    let cases = [
        ("Host a\n  HostName a.lan\n  User me\n  Port 2200\n", false),
        ("Host a\n  HostName a.lan\n\nHost b\n  HostName b\n", false),
        ("Host a\n  HostName a.lan\n  IdentityFile ~/.ssh/a\n", true),
        ("Host a\n  # home server\n  HostName a.lan\n", true),
        ("Host a b\n  HostName a.lan\n", true),
        ("Host a *.lan\n  User me\n", true),
        ("Host a\n  User me\nHost a\n  ForwardAgent yes\n", true),
    ];
    for (text, by_hand) in cases {
        assert_eq!(written_by_hand(text, "a"), by_hand, "{text}");
    }
}

#[test]
fn a_failed_write_leaves_the_old_file() {
    let folder = std::env::temp_dir().join(format!("trame-ssh-forget-{}", std::process::id()));
    let path = folder.join("config");
    let text = "Host a\n    HostName a.example\n";
    fs::create_dir_all(folder.join("config.tmp")).expect("a folder in the way");
    fs::write(&path, text).expect("written");
    let removed = SshConfig::at(&path).remove("a");
    let left = fs::read_to_string(&path).expect("read");
    fs::remove_dir_all(&folder).expect("the test folder is removed");
    assert!(removed.is_err());
    assert_eq!(left, text);
}
