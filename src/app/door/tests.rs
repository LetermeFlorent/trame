use std::sync::mpsc;
use std::time::Duration;

use super::{OPEN, heard, knock, listen};
use crate::app::scratch::Scratch;

const PATIENCE: Duration = Duration::from_secs(5);

#[test]
fn only_the_open_message_is_heard() {
    assert!(heard(OPEN));
    assert!(heard(&b"open\nand more"[..]));
    assert!(!heard(&b"close\n"[..]));
    assert!(!heard(&b"open"[..]));
    assert!(!heard(&b""[..]));
}

#[test]
fn nobody_answers_a_door_nobody_listens_at() {
    let scratch = Scratch::new("door-closed");
    assert!(!knock(scratch.path()));
}

#[test]
fn a_knock_reaches_the_listener_every_time() {
    let scratch = Scratch::new("door-open");
    let (sender, receiver) = mpsc::channel();
    listen(scratch.path(), move || {
        sender.send(()).expect("test receiver")
    });
    for _ in 0..2 {
        assert!(knock(scratch.path()));
        receiver
            .recv_timeout(PATIENCE)
            .expect("the listener heard the knock");
    }
}
