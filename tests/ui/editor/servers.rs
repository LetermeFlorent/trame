use trame::geometry::Point;
use trame::preferences::{ConnectForm, SshHost};
use trame::ui::FrameLayout;
use trame::ui::Hit;
use trame::ui::frame::{EditorFrame, HostList, ListControl};

use super::form::with_hosts;
use crate::{text_system, workspace};

fn hosts(names: &[&str]) -> Vec<SshHost> {
    names
        .iter()
        .map(|name| SshHost::parse(name).expect("a host"))
        .collect()
}

fn listing(names: &[&str]) -> FrameLayout {
    let mut text = text_system();
    with_hosts(&mut text, &workspace(), &ConnectForm::new(), &hosts(names))
}

fn editor_of(frame: &FrameLayout) -> &EditorFrame {
    frame.editor.as_ref().expect("an editor")
}

fn list_of(editor: &EditorFrame) -> &HostList {
    editor.list.as_ref().expect("a list")
}

#[test]
fn every_known_server_is_listed_above_the_form() {
    let frame = listing(&["root@10.0.0.2", "tdb"]);
    let editor = editor_of(&frame);
    let form = editor.form.as_ref().expect("a form");
    let list = list_of(editor);
    assert_eq!(list.rows.len(), 2);
    assert_eq!(editor.sections.len(), 2);
    for pair in list.rows.windows(2) {
        assert!(pair[0].line.rect.bottom() <= pair[1].line.rect.y);
    }
    let last = list.rows.last().expect("a host line");
    assert!(last.line.rect.bottom() <= form.rect.y);
    assert!(list.bar.is_none());
}

#[test]
fn a_click_on_a_listed_server_names_its_place_in_the_list() {
    let frame = listing(&["a", "b", "c"]);
    for (index, row) in list_of(editor_of(&frame)).rows.iter().enumerate() {
        let name = row.line.rect.x + 2.0;
        let point = Point::new(name, row.line.rect.center().y);
        assert_eq!(
            frame.hit(point, false),
            Some(Hit::List(ListControl::Pick(index)))
        );
    }
}

#[test]
fn each_listed_server_has_a_pencil_and_no_way_to_be_forgotten_here() {
    let frame = listing(&["a", "b"]);
    for (index, row) in list_of(editor_of(&frame)).rows.iter().enumerate() {
        assert!(row.forget.is_none());
        assert!(row.line.rect.contains(row.change.center()));
        assert_eq!(
            frame.hit(row.change.center(), false),
            Some(Hit::List(ListControl::Change(index)))
        );
    }
}

#[test]
fn without_servers_the_window_holds_only_the_form() {
    let frame = listing(&[]);
    let editor = editor_of(&frame);
    assert!(editor.list.is_none() && editor.sections.is_empty());
    assert!(editor.form.is_some());
}

#[test]
fn a_long_list_is_cut_to_what_the_window_can_show_and_scrolls() {
    let names: Vec<String> = (0..200).map(|index| format!("host{index}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let frame = listing(&names);
    let editor = editor_of(&frame);
    let popup = editor.popup.as_ref().expect("a popup").rect;
    let list = list_of(editor);
    assert!(!list.rows.is_empty() && list.rows.len() < names.len());
    let bar = list.bar.expect("a scroll bar");
    assert_eq!(bar.shown, list.rows.len());
    assert_eq!(bar.last_first, names.len() - bar.shown);
    assert_eq!(
        frame.hit(bar.thumb.center(), false),
        Some(Hit::List(ListControl::Thumb))
    );
    assert!(popup.bottom() <= frame.bounds.bottom());
    assert!(popup.y >= frame.title_bar.rect.bottom());
}
