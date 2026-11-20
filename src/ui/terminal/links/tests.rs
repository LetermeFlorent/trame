use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::LinkKind;
use super::chain::chain_links_with;
use super::paths::{find_image_paths, find_image_paths_with};
use super::resolve::{file_uri_path, resolve_image_with};
use crate::term::Terminal;

#[test]
fn osc8_file_link_caches_resolution_across_cells_and_wrapped_rows() {
    let mut term = Terminal::new(4, 12, 0);
    term.feed(b"\x1b]8;;file:///C:/pics/shot.png\x07click_wrapped_link\x1b]8;;\x07");
    let stats = AtomicUsize::new(0);
    let (_, spans) = chain_links_with(term.screen(), 0, None, &mut |_| {
        stats.fetch_add(1, Ordering::Relaxed);
        true
    });
    assert_eq!(stats.load(Ordering::Relaxed), 1);
    assert_eq!(spans.len(), 2);
    assert!(spans.iter().all(|s| s.kind == LinkKind::Image));
}

#[test]
fn unc_paths_skip_filesystem_stat_calls() {
    let stats = AtomicUsize::new(0);
    let check = |_: &Path| {
        stats.fetch_add(1, Ordering::Relaxed);
        true
    };
    assert!(resolve_image_with(r"\\server\share\a.png", None, check).is_none());
    assert!(resolve_image_with("file:////server/share/a.png", None, check).is_none());
    let unc_cwd = Path::new(r"\\server\share");
    assert!(resolve_image_with("a.png", Some(unc_cwd), check).is_none());
    assert_eq!(stats.load(Ordering::Relaxed), 0);

    let chars: Vec<char> = r"open \\server\share\a b.png now".chars().collect();
    let mut claimed = Vec::new();
    assert!(find_image_paths(&chars, &mut claimed, None).is_empty());
}

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn chain_links_sorts_earlier_row_image_before_wrapped_web_link() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = TempDir(std::env::temp_dir().join(format!("trame-chain-{stamp}")));
    fs::create_dir_all(&dir.0).expect("mkdir");
    fs::write(dir.0.join("a.png"), b"png").expect("write");
    let mut term = Terminal::new(4, 20, 0);
    term.feed(b"a.png https://a.co/1234567890");
    let (_, spans) = chain_links_with(term.screen(), 0, Some(&dir.0), &mut |_| false);
    let order: Vec<_> = spans.iter().map(|s| (s.row, s.start, s.kind)).collect();
    assert_eq!(
        order,
        [
            (0, 0, LinkKind::Image),
            (0, 6, LinkKind::Web),
            (1, 0, LinkKind::Web),
        ]
    );
}

#[test]
fn file_uri_does_not_fall_back_to_cwd_when_host_is_rejected() {
    let mut calls = 0;
    let check = |_: &Path| {
        calls += 1;
        true
    };
    let hit = resolve_image_with("FILE://remote/a.png", Some(Path::new(".")), check);
    assert!(hit.is_none());
    assert_eq!(calls, 0);
}

#[test]
fn tilde_paths_strip_extra_slashes_and_reject_empty_or_prefixed_tails() {
    let Some(home) = crate::files::home_dir() else {
        return;
    };
    let want = home.join("a.png");
    let ok = |p: &Path| p == want;
    assert_eq!(resolve_image_with("~//a.png", None, ok), Some(want.clone()));
    assert_eq!(resolve_image_with("~/\\a.png", None, ok), Some(want));
    assert!(resolve_image_with("~/", None, |_| true).is_none());
    assert!(resolve_image_with("~///", None, |_| true).is_none());
    if cfg!(windows) {
        assert!(resolve_image_with("~/C:/a.png", None, |_| true).is_none());
        assert!(resolve_image_with("~/C:a.png", None, |_| true).is_none());
    }
}

#[test]
fn pick_candidate_skips_duplicate_start_indices() {
    let mut calls = 0;
    let chars: Vec<char> = r#""/tmp/missing.png" /var/missing.png"#.chars().collect();
    let mut claimed = Vec::new();
    let found = find_image_paths_with(&chars, &mut claimed, Some(Path::new(".")), &mut |_| {
        calls += 1;
        false
    });
    assert!(found.is_empty());
    assert_eq!(calls, 2);
}

#[test]
fn after_delim_retries_parens_and_skips_scheme_slashes() {
    let cwd = Some(Path::new("."));
    let a: Vec<char> = "![alt](shot(1).png) img=shot(1).png".chars().collect();
    let mut ok = |p: &Path| p.file_name().is_some_and(|n| n == "shot(1).png");
    let found = find_image_paths_with(&a, &mut Vec::new(), cwd, &mut ok);
    let spans: Vec<_> = found.iter().map(|f| (f.start, f.end)).collect();
    assert_eq!(spans, [(7, 18), (24, 35)]);
    let raw = "img=file://remote/a.png s3://bucket/a.png";
    let b: Vec<char> = raw.chars().collect();
    let mut bad_probe = false;
    let mut spy = |p: &Path| {
        bad_probe |= p == Path::new("//remote/a.png") || p == Path::new("//bucket/a.png");
        false
    };
    assert!(find_image_paths_with(&b, &mut Vec::new(), cwd, &mut spy).is_empty());
    assert!(!bad_probe);
}

#[test]
fn windows_drive_relative_and_file_uri_platform_gating() {
    let mut calls = 0;
    let hit = resolve_image_with("C:a.png", Some(Path::new(".")), |_| {
        calls += 1;
        true
    });
    if cfg!(windows) {
        assert!(hit.is_none() && calls == 0);
    }
    let idx = usize::from(cfg!(windows));
    let want = Path::new(["/C:/a.png", "C:/a.png"][idx]);
    assert_eq!(file_uri_path("file:///C:/a.png").as_deref(), Some(want));
}
