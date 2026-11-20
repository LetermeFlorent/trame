use crate::feed;

fn attention(input: &str) -> Option<String> {
    feed(1, 10, input).screen_mut().take_attention()
}

#[test]
fn a_bell_asks_for_attention_without_a_message() {
    assert_eq!(attention("ok\x07").as_deref(), Some(""));
    assert_eq!(attention("ok"), None);
}

#[test]
fn notification_sequences_carry_their_message() {
    assert_eq!(
        attention("\x1b]9;Need input\x07").as_deref(),
        Some("Need input")
    );
    let notify = attention("\x1b]777;notify;Claude;Waiting\x1b\\");
    assert_eq!(notify.as_deref(), Some("Waiting"));
    assert_eq!(attention("\x1b]99;i=1;Done\x07").as_deref(), Some("Done"));
}

#[test]
fn progress_and_title_sequences_stay_silent() {
    assert_eq!(attention("\x1b]9;4;1;50\x07"), None);
    assert_eq!(attention("\x1b]9;9;C:\\x07"), None);
    assert_eq!(attention("\x1b]0;title\x07"), None);
}
