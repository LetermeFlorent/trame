use crate::feed;

fn responses(input: &str) -> String {
    let mut term = feed(5, 10, input);
    String::from_utf8(term.screen_mut().take_responses()).unwrap()
}

#[test]
fn a_console_reading_win32_input_gets_the_status_reply_as_key_presses() {
    let reply = responses("\x1b[?9001h\x1b]7501;?\x07");
    let pressed: String = reply
        .split_terminator('_')
        .filter_map(|event| event.strip_prefix("\x1b[0;0;")?.strip_suffix(";1;0;1"))
        .map(|unit| char::from_u32(unit.parse().unwrap()).unwrap())
        .collect();
    assert_eq!(
        pressed,
        "\x1b]7501;?:states=idle,working,done,blocked,error:kinds=permission,question,auth\x07"
    );
    assert_eq!(reply.matches(";0;0;1_").count(), pressed.len());
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
