use crate::feed;

fn responses(input: &str) -> String {
    let mut term = feed(5, 10, input);
    String::from_utf8(term.screen_mut().take_responses()).unwrap()
}

#[test]
fn reports_cursor_position() {
    assert_eq!(responses("\x1b[3;4H\x1b[6n"), "\x1b[3;4R");
}

#[test]
fn reports_status_and_attributes() {
    assert_eq!(responses("\x1b[5n"), "\x1b[0n");
    assert_eq!(responses("\x1b[c"), "\x1b[?62;22c");
    assert!(responses("\x1b[>c").starts_with("\x1b[>"));
}

#[test]
fn reports_mode_state() {
    assert_eq!(responses("\x1b[?2026$p"), "\x1b[?2026;2$y");
    assert_eq!(responses("\x1b[?2026h\x1b[?2026$p"), "\x1b[?2026;1$y");
    assert_eq!(responses("\x1b[?9999$p"), "\x1b[?9999;0$y");
}

#[test]
fn reports_colors_with_matching_terminator() {
    let reply = responses("\x1b]11;?\x07");
    assert_eq!(reply, "\x1b]11;rgb:2828/2c2c/3333\x07");
    let reply = responses("\x1b]10;?\x1b\\");
    assert!(reply.ends_with("\x1b\\"));
}

#[test]
fn reports_text_area_size() {
    assert_eq!(responses("\x1b[18t"), "\x1b[8;5;10t");
}
