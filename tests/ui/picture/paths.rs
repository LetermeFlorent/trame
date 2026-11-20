use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use trame::files::encode_png;
use trame::files::pictures::{decode, image_limits};
use trame::term::Terminal;
use trame::ui::terminal::links::{LinkKind, link_at, link_hit, view_links, view_links_in};

const WEBP_1X1: &[u8] = &[
    82, 73, 70, 70, 30, 0, 0, 0, 87, 69, 66, 80, 86, 80, 56, 76, 17, 0, 0, 0, 47, 0, 0, 0, 0, 7,
    80, 198, 82, 84, 191, 255, 129, 136, 232, 127, 0, 0,
];

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
fn webp_bytes_decode_under_limits() {
    let img = decode(WEBP_1X1, image_limits()).expect("webp decode");
    assert_eq!(img.dimensions(), (1, 1));
    assert_eq!(img.get_pixel(0, 0).0, [20, 140, 250, 255]);
}

#[test]
fn local_image_paths_are_detected_and_web_urls_stay_web() {
    let dir = TempDir::new("links");
    let sub = dir.0.join("My Captures");
    fs::create_dir_all(&sub).expect("sub dir");
    let (shot, webp) = (sub.join("capture-1.png"), dir.0.join("pic.WEBP"));
    let missing = dir.0.join("missing.png");
    fs::write(&shot, encode_png(2, 2, &[255; 16]).expect("png")).expect("write png");
    fs::write(&webp, WEBP_1X1).expect("write webp");
    let mut term = Terminal::new(8, 120, 0);
    let cwd = dir.0.to_string_lossy().replace('\\', "/");
    let cwd = cwd.trim_start_matches('/');
    term.feed(format!("\x1b]7;file:///{cwd}\x07").as_bytes());
    let (s, m) = (shot.display(), missing.display());
    term.feed(
        format!("\"{s}\"\r\n{s}\r\nhttps://example.com/a.png\r\n{m}\r\npic.WEBP\r\n").as_bytes(),
    );
    let screen = term.screen();
    let spans = view_links(screen, 0..6);
    let has = |k, t: &str| {
        spans
            .iter()
            .any(|sp| sp.kind == k && Path::new(&sp.target) == Path::new(t))
    };
    assert!(has(LinkKind::Image, &shot.to_string_lossy()));
    assert!(has(LinkKind::Image, &webp.to_string_lossy()));
    assert!(
        spans
            .iter()
            .any(|sp| sp.kind == LinkKind::Web && sp.target == "https://example.com/a.png")
    );
    assert!(!spans.iter().any(|sp| sp.target.contains("missing.png")));
    let shot_s = shot.to_string_lossy();
    assert_eq!(link_at(screen, 0, 2).as_deref(), Some(&*shot_s));
    assert_eq!(link_at(screen, 1, 2).as_deref(), Some(&*shot_s));
    let hit = link_hit(screen, 4, 1, None).expect("relative webp hit");
    assert_eq!(hit.kind, LinkKind::Image);
    assert_eq!(Path::new(&hit.target), webp);
}

#[test]
fn wrapped_jpg_and_jpeg_paths_are_detected() {
    let dir = TempDir::new("links-wrap");
    let sub = dir.0.join("My Captures");
    fs::create_dir_all(&sub).expect("sub dir");
    let jpg = sub.join("a b.jpg");
    let jpeg = dir.0.join("photo.jpeg");
    fs::write(&jpg, b"jpg").expect("write jpg");
    fs::write(&jpeg, b"jpeg").expect("write jpeg");
    let mut wrapped = Terminal::new(16, 20, 0);
    wrapped.feed(format!("{}\r\n{}", jpg.display(), jpeg.display()).as_bytes());
    let spans = view_links(wrapped.screen(), 0..16);
    let count = |p: &Path| {
        spans
            .iter()
            .filter(|sp| sp.kind == LinkKind::Image && Path::new(&sp.target) == p)
            .count()
    };
    assert!(count(&jpg) >= 2, "{spans:?}");
    assert!(count(&jpeg) >= 1, "{spans:?}");
}

#[test]
fn file_uri_with_drive_and_spaces_keeps_file_scheme_intact() {
    let dir = TempDir::new("file-uri");
    let sub = dir.0.join("My Pics");
    fs::create_dir_all(&sub).expect("sub dir");
    let img = sub.join("a b.png");
    fs::write(&img, b"png").expect("write png");
    let slash = dir.0.to_string_lossy().replace('\\', "/");
    let rel = slash.strip_prefix('/').unwrap_or(&slash);
    let uri = format!("file:///{rel}/My%20Pics/a b.png");
    let mut term = Terminal::new(4, 160, 0);
    term.feed(uri.as_bytes());
    let spans = view_links(term.screen(), 0..4);
    assert_eq!(spans.len(), 1, "{spans:?}");
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, uri.chars().count());
    assert_eq!(spans[0].kind, LinkKind::Image);
    assert_eq!(Path::new(&spans[0].target), img);
}

#[test]
fn delimited_image_tokens_resolve_after_delimiter() {
    let dir = TempDir::new("delim");
    fs::write(dir.0.join("a.png"), b"png").unwrap();
    fs::write(dir.0.join("shot(1).png"), b"png").unwrap();
    let mut term = Terminal::new(2, 100, 0);
    term.feed(b"img=a.png ![alt](a.png) file:a.png {a.png} shot(1).png");
    let spans = view_links_in(term.screen(), 0..2, Some(&dir.0));
    let got: Vec<_> = spans.iter().map(|s| (s.start, s.end)).collect();
    assert_eq!(got, [(4, 9), (17, 22), (29, 34), (36, 41), (43, 54)]);
}

#[test]
fn non_hex_percent_escape_stays_literal_in_file_uri() {
    let dir = TempDir::new("pct");
    let img = dir.0.join("%+1.png");
    fs::write(&img, b"png").unwrap();
    let slash = dir.0.to_string_lossy().replace('\\', "/");
    let uri = format!("file:///{}/%+1.png", slash.trim_start_matches('/'));
    let mut term = Terminal::new(2, 160, 0);
    term.feed(uri.as_bytes());
    let spans = view_links(term.screen(), 0..2);
    assert_eq!(spans.len(), 1, "{spans:?}");
    assert_eq!(Path::new(&spans[0].target), img);
}
