use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use trame::files::pictures::{MAX_IMAGE_FILE_BYTES, read_image_bytes};
use trame::session::SessionId;
use trame::term::Terminal;
use trame::ui::terminal::links::{
    LinkCache, LinkKind, link_hit, resolve_image_path, view_links, view_links_in,
};
use trame::workspace::Workspace;

use crate::{frame, text_system};

mod cwd;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("trame-pic-{tag}-{stamp}"));
        fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn link_cache_underlines_relative_image_paths_by_cwd_and_invalidates_on_change() {
    let dir_a = TempDir::new("cwd-a");
    let dir_b = TempDir::new("cwd-b");
    let a = dir_a.0.join("a.png");
    let b = dir_b.0.join("b.png");
    fs::write(&a, b"png").expect("write a");
    fs::write(&b, b"png").expect("write b");
    let mut term = Terminal::new(4, 40, 0);
    term.feed(b"a.png b.png");
    assert!(term.screen().cwd().is_none());

    let mut cache = LinkCache::default();
    let spans_a = cache.spans(SessionId(1), &term, 0..4, Some(&dir_a.0));
    assert_eq!(spans_a.len(), 1);
    assert_eq!(Path::new(&spans_a[0].target), a);

    let spans_b = cache.spans(SessionId(1), &term, 0..4, Some(&dir_b.0));
    assert_eq!(spans_b.len(), 1);
    assert_eq!(Path::new(&spans_b[0].target), b);

    assert!(cache.spans(SessionId(1), &term, 0..4, None).is_empty());
}

#[test]
fn osc8_file_image_link_spans_wrapped_rows_and_unc_stays_empty() {
    let dir = TempDir::new("osc8-img");
    let img = dir.0.join("shot.png");
    fs::write(&img, b"png").expect("write png");
    let slash = img.to_string_lossy().replace('\\', "/");
    let rel = slash.strip_prefix('/').unwrap_or(&slash);
    let input = format!("\x1b]8;;file:///{rel}\x07wrapped_osc8_link\x1b]8;;\x07");
    let mut term = Terminal::new(4, 10, 0);
    term.feed(input.as_bytes());
    let spans = view_links(term.screen(), 0..4);
    assert_eq!(spans.len(), 2);
    assert!(spans.iter().all(|s| s.kind == LinkKind::Image));
    let hit = link_hit(term.screen(), 1, 2, None).expect("wrapped hit");
    assert_eq!(Path::new(&hit.target), img);

    let mut unc_term = Terminal::new(2, 60, 0);
    unc_term.feed(br"\\nonexistent-trame-host\share\a b.png");
    assert!(view_links(unc_term.screen(), 0..2).is_empty());
}

#[test]
fn read_image_bytes_opens_once_and_enforces_file_and_size_limits() {
    let dir = TempDir::new("read-bytes");
    let ok = dir.0.join("ok.png");
    fs::write(&ok, b"hello").expect("write ok");
    assert_eq!(read_image_bytes(&ok).expect("read ok"), b"hello");
    assert!(read_image_bytes(&dir.0).is_err());

    let big = dir.0.join("big.png");
    let file = File::create(&big).expect("create big");
    file.set_len(MAX_IMAGE_FILE_BYTES + 1).expect("set len");
    assert!(read_image_bytes(&big).is_err());
}

#[test]
fn tilde_prefixed_or_rooted_tail_does_not_escape_home() {
    let dir = TempDir::new("tilde-escape");
    let img = dir.0.join("a.png");
    fs::write(&img, b"png").expect("write png");
    let slash = img.to_string_lossy().replace('\\', "/");
    let raw = format!("~/{slash}");
    assert!(resolve_image_path(&raw, None).is_none(), "{raw}");
}

#[test]
fn brace_and_semicolon_delimited_image_paths_resolve() {
    let dir = TempDir::new("brace-semi");
    let sub = dir.0.join("My Pics");
    fs::create_dir_all(&sub).expect("sub dir");
    let nested = sub.join("a.png");
    let paren = dir.0.join("a(1).png");
    fs::write(&nested, b"png").expect("write nested");
    fs::write(&paren, b"png").expect("write paren");
    let mut term = Terminal::new(2, 160, 0);
    let line = format!("{{a(1).png}} ;a(1).png {{{}}}", nested.display());
    term.feed(line.as_bytes());
    let spans = view_links_in(term.screen(), 0..2, Some(&dir.0));
    assert_eq!(spans.len(), 3, "{spans:?}");
}

#[test]
fn draw_frame_uses_session_cwd_when_provider_is_none_and_overrides_when_some() {
    let dir_a = TempDir::new("cwd-none");
    let dir_b = TempDir::new("cwd-some");
    fs::write(dir_a.0.join("a.png"), b"png").expect("png a");
    fs::write(dir_b.0.join("b.png"), b"png").expect("png b");
    let id = SessionId(1);
    let session = cwd::silent_session(id);
    let slash = dir_a.0.to_string_lossy().replace('\\', "/");
    let rel = slash.trim_start_matches('/');
    let osc7 = format!("\x1b]7;file:///{rel}\x07");
    session.terminal().feed(osc7.as_bytes());
    assert!(session.cwd().is_some());
    let mut map = HashMap::from([(id, session)]);
    let mut ws = Workspace::new();
    ws.add_instance(None, id);
    let mut text = text_system();
    let layout = frame(&mut text, &ws);
    assert_eq!(cwd::underline_count(&mut text, &layout, &map, None), 5);
    assert_eq!(
        cwd::underline_count(&mut text, &layout, &map, Some(&|_| Some(dir_b.0.clone()))),
        0
    );
    map.clear();
}
