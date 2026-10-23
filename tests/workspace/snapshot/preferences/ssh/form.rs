use trame::preferences::{ConnectForm, FormField, HostEntry, SshHost};

use super::host;

fn filled(address: &str, user: &str, port: &str, secret: &str) -> ConnectForm {
    let mut form = ConnectForm::new();
    for (field, text) in FormField::ALL
        .into_iter()
        .zip([address, user, port, secret])
    {
        form.focus_on(field);
        form.focused_mut().insert(text);
    }
    form
}

#[test]
fn the_parts_of_a_connection_build_its_host() {
    let cases = [
        (
            ("10.0.0.2", "root", "2222"),
            host("root@10.0.0.2", Some(2222)),
        ),
        ((" box.lan ", " me ", " 22 "), host("me@box.lan", None)),
        (("syllabis-vps", "", ""), host("syllabis-vps", None)),
        (("me@box:2200", "", ""), host("me@box", Some(2200))),
        (("box:2200", "me", "2300"), host("me@box", Some(2300))),
    ];
    for ((address, user, port), expected) in cases {
        let built = SshHost::from_parts(address, user, port);
        assert_eq!(built, Some(expected), "{address} {user} {port}");
    }
}

#[test]
fn broken_parts_build_no_host() {
    let cases = [
        ("", "root", ""),
        ("box", "", "abc"),
        ("box", "", "0"),
        ("box", "", "70000"),
        ("me@box", "you", ""),
        ("box", "a b", ""),
        ("-oProxyCommand=x", "", ""),
        ("box", "-l", ""),
    ];
    for (address, user, port) in cases {
        assert_eq!(
            SshHost::from_parts(address, user, port),
            None,
            "{address} {user} {port}"
        );
    }
}

#[test]
fn tab_walks_the_fields_in_a_loop() {
    let mut form = ConnectForm::new();
    assert_eq!(form.focus(), FormField::Address);
    let walked: Vec<FormField> = (0..4)
        .map(|_| {
            form.next();
            form.focus()
        })
        .collect();
    let expected = [
        FormField::User,
        FormField::Port,
        FormField::Secret,
        FormField::Address,
    ];
    assert_eq!(walked, expected);
    form.previous();
    assert_eq!(form.focus(), FormField::Secret);
}

#[test]
fn the_form_types_into_the_focused_field_only() {
    let form = filled("box", "me", "2222", "hunter2");
    assert_eq!(form.field(FormField::Address).text(), "box");
    assert_eq!(form.field(FormField::User).text(), "me");
    assert_eq!(form.host(), Some(host("me@box", Some(2222))));
    assert_eq!(form.secret(), Some("hunter2"));
    assert_eq!(filled("box", "", "", "").secret(), None);
}

#[test]
fn the_first_broken_field_is_the_one_to_fix() {
    let cases = [
        (filled("", "me", "x", ""), Some(FormField::Address)),
        (filled("box", "a b", "x", ""), Some(FormField::User)),
        (filled("box", "me", "x", ""), Some(FormField::Port)),
        (filled("box", "me", "22", ""), None),
    ];
    for (form, expected) in cases {
        assert_eq!(form.first_invalid(), expected);
    }
}

#[test]
fn the_debug_view_of_a_form_never_shows_the_secret() {
    let form = filled("box", "me", "", "hunter2");
    assert!(!format!("{form:?}").contains("hunter2"));
}

#[test]
fn a_known_connection_fills_the_form_to_change_it() {
    let entry = HostEntry {
        address: "10.0.0.2".to_owned(),
        user: Some("root".to_owned()),
        port: Some(2222),
    };
    let form = ConnectForm::filled(&entry, Some("pass"));
    assert_eq!(form.host(), Some(host("root@10.0.0.2", Some(2222))));
    assert_eq!(form.secret(), Some("pass"));
    assert_eq!(form.focus(), FormField::Address);
    let bare = ConnectForm::filled(&HostEntry::of(&host("box", None)), None);
    assert_eq!(bare.field(FormField::User).text(), "");
    assert_eq!(bare.field(FormField::Port).text(), "");
    assert_eq!(bare.secret(), None);
}
