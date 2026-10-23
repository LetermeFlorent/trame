use std::fs;

use trame::preferences::{SshConfig, SshHost, aliases_in, appended, menu_hosts};

use super::host;

const CONFIG: &str = "\u{feff}# servers\n\
Host syllabis-vps\n    HostName 31.97.199.70\n    User deploy\n\
host=growthfit growthfit-root # two names\n\
Host *.lan !secret office?\n\
Match host tiky\n    User root\n\
Host = tdb\n\
Host syllabis-vps\n\
Host me@box box:22 -oProxyCommand\n";

#[test]
fn only_plain_host_names_are_found() {
    assert_eq!(
        aliases_in(CONFIG),
        ["syllabis-vps", "growthfit", "growthfit-root", "tdb"]
    );
    assert!(aliases_in("").is_empty());
}

#[test]
fn a_new_entry_follows_the_file_with_a_blank_line() {
    let entry = appended("Host old\n", &host("root@10.0.0.2", Some(2222)));
    assert_eq!(
        entry,
        "\nHost 10.0.0.2\n    HostName 10.0.0.2\n    User root\n    Port 2222\n"
    );
    let bare = appended("", &host("box.lan", None));
    assert_eq!(bare, "Host box.lan\n    HostName box.lan\n");
    let unfinished = appended("Host old", &host("box", None));
    assert!(unfinished.starts_with("\n\nHost box\n"));
}

#[test]
fn a_windows_file_keeps_its_line_endings() {
    let entry = appended("Host old\r\n", &host("me@box", None));
    assert_eq!(entry, "\r\nHost box\r\n    HostName box\r\n    User me\r\n");
}

#[test]
fn the_menu_lists_recent_hosts_then_the_new_ones_from_the_config() {
    let saved = [host("root@10.0.0.2", None), host("tdb", None)];
    let found = ["tdb", "growthfit"].map(str::to_owned);
    assert_eq!(
        menu_hosts(&saved, &found),
        [
            host("root@10.0.0.2", None),
            host("tdb", None),
            host("growthfit", None)
        ]
    );
}

#[test]
fn a_kept_connection_is_written_once_and_used_by_its_name() {
    let folder = std::env::temp_dir().join(format!("trame-ssh-config-{}", std::process::id()));
    let config = SshConfig::at(folder.join(".ssh").join("config"));
    assert_eq!(config.aliases().expect("a missing file is empty"), [""; 0]);
    let typed = host("root@10.0.0.2", Some(2222));
    let kept = config.keep(&typed).expect("the entry is written");
    assert_eq!(kept, host("10.0.0.2", None));
    assert_eq!(config.keep(&typed).expect("nothing to add"), typed);
    assert_eq!(config.keep(&kept).expect("nothing to add"), kept);
    assert_eq!(config.aliases().expect("the file is read"), ["10.0.0.2"]);
    let text = fs::read_to_string(folder.join(".ssh").join("config")).expect("the file exists");
    assert_eq!(text.matches("Host 10.0.0.2").count(), 1);
    fs::remove_dir_all(&folder).expect("the test folder is removed");
    assert_eq!(SshHost::parse("10.0.0.2"), Some(kept));
}
