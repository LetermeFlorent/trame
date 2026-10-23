use std::fs;

use trame::session::{MAX_TRANSCRIPT_BYTES, Transcripts};

use super::{FIRST, SECOND, Sandbox, key_of};

#[test]
fn unsafe_keys_are_refused() {
    let sandbox = Sandbox::new("keys");
    fs::create_dir_all(sandbox.text()).unwrap();
    fs::write(sandbox.root.join("escape.vt"), b"outside").unwrap();
    let mut transcripts = sandbox.transcripts();
    let too_long = "a".repeat(41);
    for key in ["", "../escape", "a/b", "a.b", too_long.as_str()] {
        assert_eq!(transcripts.read(key), None, "{key:?}");
        transcripts.adopt(FIRST, Some(key));
        assert_eq!(transcripts.key(FIRST), None, "{key:?}");
    }
    let longest = "a".repeat(40);
    transcripts.adopt(FIRST, Some(&longest));
    assert_eq!(transcripts.key(FIRST), Some(longest.as_str()));
}

#[test]
fn adopted_keys_survive_assign() {
    let mut transcripts = Transcripts::new(None);
    transcripts.adopt(FIRST, Some("saved-1"));
    transcripts.assign(&[FIRST, SECOND]);
    assert_eq!(transcripts.key(FIRST), Some("saved-1"));
    let fresh = key_of(&transcripts, SECOND);
    assert!(fresh.ends_with("-2"), "{fresh}");
}

#[test]
fn oversized_files_are_ignored() {
    let sandbox = Sandbox::new("oversized");
    fs::create_dir_all(sandbox.text()).unwrap();
    fs::write(
        sandbox.text().join("big.vt"),
        vec![b'x'; MAX_TRANSCRIPT_BYTES + 1],
    )
    .unwrap();
    assert_eq!(sandbox.transcripts().read("big"), None);
}
