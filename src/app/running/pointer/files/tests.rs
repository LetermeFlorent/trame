use std::path::{Path, PathBuf};

use trame::ui::Hit;
use trame::workspace::PaneId;

use super::target::{DropTarget, drop_target};
use super::text::dropped_text;

fn text_of(paths: &[&str]) -> String {
    let owned: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    dropped_text(owned.iter().map(PathBuf::as_path))
}

#[test]
fn writes_the_path_as_given_with_one_space_after() {
    assert_eq!(
        text_of(&[r"C:\Users\me\shot.png"]),
        r"C:\Users\me\shot.png "
    );
}

#[test]
fn never_quotes_a_path_holding_spaces() {
    let path = r"C:\Users\me\My Pictures\a b.png";
    assert_eq!(text_of(&[path]), format!("{path} "));
}

#[test]
fn keeps_the_drop_order_of_several_files() {
    assert_eq!(
        text_of(&["/tmp/b.txt", "/tmp/a.txt"]),
        "/tmp/b.txt /tmp/a.txt "
    );
}

#[test]
fn skips_a_path_holding_a_control_character() {
    let hostile = [
        "/tmp/a\nrm -rf ~",
        "/tmp/\x1b[201~x",
        "/tmp/\u{7f}",
        "/tmp/\u{9b}x",
    ];
    for path in hostile {
        assert_eq!(text_of(&[path, "/tmp/ok"]), "/tmp/ok ");
    }
}

#[test]
fn strips_the_windows_verbatim_prefix() {
    assert_eq!(text_of(&[r"\\?\C:\long\file.txt"]), r"C:\long\file.txt ");
    assert_eq!(
        text_of(&[r"\\?\UNC\server\share\file.txt"]),
        r"\\server\share\file.txt "
    );
    assert_eq!(text_of(&[r"\\server\share\x"]), r"\\server\share\x ");
}

#[test]
fn skips_an_empty_path() {
    assert_eq!(dropped_text([Path::new("")]), "");
}

#[cfg(windows)]
fn not_unicode() -> PathBuf {
    use std::os::windows::ffi::OsStringExt;
    PathBuf::from(std::ffi::OsString::from_wide(&[0x43, 0xD800]))
}

#[cfg(unix)]
fn not_unicode() -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    PathBuf::from(std::ffi::OsStr::from_bytes(b"/tmp/\xff"))
}

#[test]
fn skips_a_path_that_is_not_unicode() {
    let kept = PathBuf::from("/tmp/ok");
    let text = dropped_text([not_unicode().as_path(), kept.as_path()]);
    assert_eq!(text, "/tmp/ok ");
}

#[test]
fn types_into_the_pane_under_the_point() {
    let hit = Some(Hit::Terminal(PaneId(3)));
    assert_eq!(drop_target(hit, false), DropTarget::Pane(PaneId(3)));
}

#[test]
fn falls_back_to_the_focused_pane_outside_a_terminal() {
    let outside = [
        None,
        Some(Hit::TitleBar),
        Some(Hit::TabBar(PaneId(3))),
        Some(Hit::Sidebar),
    ];
    for hit in outside {
        assert_eq!(drop_target(hit, false), DropTarget::Focused);
    }
}

#[test]
fn ignores_the_drop_while_a_modal_is_open() {
    let hit = Some(Hit::Terminal(PaneId(3)));
    assert_eq!(drop_target(hit, true), DropTarget::Ignored);
    assert_eq!(drop_target(None, true), DropTarget::Ignored);
}
