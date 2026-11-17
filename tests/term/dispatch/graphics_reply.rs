use crate::feed;

const DA1: &str = "\x1b[?62;22c";

fn responses(input: &str) -> String {
    let mut term = feed(5, 10, input);
    String::from_utf8(term.screen_mut().take_responses()).unwrap()
}

#[test]
fn answers_the_opentui_probe_before_the_device_attributes() {
    let reply = responses("\x1b_Gi=31337,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c");
    assert_eq!(reply, format!("\x1b_Gi=31337;OK\x1b\\{DA1}"));
}

#[test]
fn answers_the_claude_probe_before_the_device_attributes() {
    let reply = responses("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c");
    assert_eq!(reply, format!("\x1b_Gi=31;OK\x1b\\{DA1}"));
}

#[test]
fn a_query_stores_nothing() {
    let term = feed(5, 10, "\x1b_Gi=31,s=1,v=1,a=q,f=24;AAAA\x1b\\");
    assert!(term.screen().graphics().get(31).is_none());
}

#[test]
fn quiet_two_silences_every_reply() {
    assert_eq!(responses("\x1b_Gi=31,s=1,v=1,a=q,f=24,q=2;AAAA\x1b\\"), "");
    assert_eq!(responses("\x1b_Gi=31,s=2,v=1,a=q,f=24,q=2;AAAA\x1b\\"), "");
}

#[test]
fn quiet_one_keeps_only_errors() {
    assert_eq!(responses("\x1b_Gi=31,s=1,v=1,a=q,f=24,q=1;AAAA\x1b\\"), "");
    assert_eq!(
        responses("\x1b_Gi=31,s=2,v=1,a=q,f=24,q=1;AAAA\x1b\\"),
        "\x1b_Gi=31;EINVAL:invalid\x1b\\"
    );
}

#[test]
fn a_wrong_rgb_length_is_invalid() {
    assert_eq!(
        responses("\x1b_Gi=7,s=2,v=1,f=24;AAAA\x1b\\"),
        "\x1b_Gi=7;EINVAL:invalid\x1b\\"
    );
}

#[test]
fn file_transport_is_refused() {
    assert_eq!(
        responses("\x1b_Gi=7,t=f,f=100;L3RtcC94LnBuZw==\x1b\\"),
        "\x1b_Gi=7;EBADF:unsupported transport\x1b\\"
    );
}

#[test]
fn an_id_and_a_number_together_are_invalid() {
    assert_eq!(
        responses("\x1b_Gi=7,I=5,s=1,v=1,f=24;AAAA\x1b\\"),
        "\x1b_Gi=7,I=5;EINVAL:invalid\x1b\\"
    );
}

#[test]
fn an_unsupported_action_is_invalid() {
    assert_eq!(
        responses("\x1b_Ga=f,i=7;AAAA\x1b\\"),
        "\x1b_Gi=7;EINVAL:invalid\x1b\\"
    );
}

#[test]
fn a_number_alone_gets_an_assigned_id() {
    let mut term = feed(5, 10, "\x1b_GI=5,s=1,v=1,f=24;AAAA\x1b\\");
    let reply = String::from_utf8(term.screen_mut().take_responses()).unwrap();
    let id: u32 = reply
        .strip_prefix("\x1b_Gi=")
        .and_then(|rest| rest.strip_suffix(",I=5;OK\x1b\\"))
        .unwrap()
        .parse()
        .unwrap();
    assert!(id >= 1 << 31);
    assert_eq!(term.screen().graphics().by_number(5), Some(id));
    assert!(term.screen().graphics().get(id).is_some());
}

#[test]
fn an_anonymous_transmit_stays_silent() {
    assert_eq!(responses("\x1b_Gs=1,v=1,f=24;AAAA\x1b\\"), "");
}
