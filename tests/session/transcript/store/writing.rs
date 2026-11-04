use std::fs;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use trame::session::{TRANSCRIPT_SAVE_INTERVAL, Transcripts};

use super::{FIRST, SECOND, Sandbox, key_of, save_now, tab_with};

#[test]
fn written_text_reads_back_and_marks_the_generation() {
    let sandbox = Sandbox::new("round-trip");
    let mut transcripts = sandbox.transcripts();
    let first = tab_with(b"hello");
    save_now(&mut transcripts, &[(FIRST, &first)]);
    let key = key_of(&transcripts, FIRST);
    assert_eq!(transcripts.read(&key), Some(b"hello\r\n".to_vec()));
    fs::remove_file(sandbox.text().join(format!("{key}.vt"))).unwrap();
    save_now(&mut transcripts, &[(FIRST, &first)]);
    assert_eq!(transcripts.read(&key), None);
    first.lock().unwrap().feed(b" again");
    save_now(&mut transcripts, &[(FIRST, &first)]);
    assert_eq!(transcripts.read(&key), Some(b"hello again\r\n".to_vec()));
}

#[test]
fn saving_removes_closed_tabs_and_stray_files() {
    let sandbox = Sandbox::new("prune");
    let mut transcripts = sandbox.transcripts();
    let (first, second) = (tab_with(b"first"), tab_with(b"second"));
    save_now(&mut transcripts, &[(FIRST, &first), (SECOND, &second)]);
    fs::write(sandbox.text().join("stray.vt"), b"old").unwrap();
    save_now(&mut transcripts, &[(FIRST, &first)]);
    let kept = format!("{}.vt", key_of(&transcripts, FIRST));
    assert_eq!(sandbox.files(), vec![kept]);
    assert_eq!(transcripts.key(SECOND), None);
}

#[test]
fn clear_removes_every_saved_text() {
    let sandbox = Sandbox::new("clear");
    let mut transcripts = sandbox.transcripts();
    let first = tab_with(b"gone");
    save_now(&mut transcripts, &[(FIRST, &first)]);
    transcripts.clear();
    assert!(!sandbox.text().exists());
    let key = key_of(&transcripts, FIRST);
    assert_eq!(transcripts.read(&key), None);
    save_now(&mut transcripts, &[(FIRST, &first)]);
    assert_eq!(transcripts.read(&key), Some(b"gone\r\n".to_vec()));
}

#[test]
fn without_a_folder_nothing_is_written() {
    let mut transcripts = Transcripts::new(None);
    save_now(&mut transcripts, &[(FIRST, &tab_with(b"lost"))]);
    assert_eq!(transcripts.read(&key_of(&transcripts, FIRST)), None);
}

#[test]
fn saving_does_not_wait_for_a_busy_tab() {
    let sandbox = Sandbox::new("busy");
    let mut transcripts = sandbox.transcripts();
    let first = tab_with(b"busy");
    let held = first.lock().unwrap();
    let (sender, receiver) = mpsc::channel();
    let returned = thread::scope(|scope| {
        scope.spawn(|| {
            transcripts.save(&[FIRST], |_| Some(Arc::clone(&first)));
            let _ = sender.send(());
        });
        let returned = receiver.recv_timeout(Duration::from_secs(5)).is_ok();
        drop(held);
        returned
    });
    assert!(returned, "saving waited for a busy tab");
    transcripts.flush();
    let key = key_of(&transcripts, FIRST);
    assert_eq!(transcripts.read(&key), Some(b"busy\r\n".to_vec()));
}

#[test]
fn first_save_waits_a_full_interval() {
    let before = Instant::now();
    let mut transcripts = Transcripts::new(None);
    assert!(transcripts.due() >= before + TRANSCRIPT_SAVE_INTERVAL);
    let rescheduled = Instant::now();
    transcripts.schedule();
    assert!(transcripts.due() >= rescheduled + TRANSCRIPT_SAVE_INTERVAL);
}

#[cfg(unix)]
#[test]
fn a_transcript_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let sandbox = Sandbox::new("private");
    let mut transcripts = sandbox.transcripts();
    save_now(&mut transcripts, &[(FIRST, &tab_with(b"secret"))]);
    let key = key_of(&transcripts, FIRST);
    let file = sandbox.text().join(format!("{key}.vt"));
    let expected = [
        (sandbox.root.clone(), 0o700),
        (sandbox.text(), 0o700),
        (file, 0o600),
    ];
    for (path, private) in expected {
        let mode = fs::metadata(path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, private);
    }
}
