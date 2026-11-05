use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use super::{OPEN, heard, knock, listen};
use crate::app::scratch::Scratch;

const PATIENCE: Duration = Duration::from_secs(5);

#[test]
fn only_the_open_message_is_heard_with_its_folder() {
    assert_eq!(heard(OPEN), Some(None));
    let folder = Some(PathBuf::from("C:/work"));
    assert_eq!(heard(&b"open\nC:/work"[..]), Some(folder));
    assert_eq!(heard(&b"close\n"[..]), None);
    assert_eq!(heard(&b"open"[..]), None);
    assert_eq!(heard(&b""[..]), None);
}

#[test]
fn nobody_answers_a_door_nobody_listens_at() {
    let scratch = Scratch::new("door-closed");
    assert!(!knock(scratch.path(), None));
}

#[test]
fn a_knock_reaches_the_listener_every_time() {
    let scratch = Scratch::new("door-open");
    let (sender, receiver) = mpsc::channel();
    listen(scratch.path(), move |opened| {
        sender.send(opened).expect("test receiver")
    });
    let work = Path::new("C:/work");
    for opened in [None, Some(work)] {
        assert!(knock(scratch.path(), opened));
        let heard = receiver
            .recv_timeout(PATIENCE)
            .expect("the listener heard the knock");
        assert_eq!(heard.as_deref(), opened);
    }
}
