use trame::preferences::{ConnectForm, SshHost};
use trame::ui::FrameLayout;
use trame::ui::Hit;
use trame::ui::frame::EditorFrame;

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

#[test]
fn every_known_server_is_listed_above_the_form() {
    let frame = listing(&["root@10.0.0.2", "tdb"]);
    let editor = editor_of(&frame);
    let form = editor.form.as_ref().expect("a form");
    assert_eq!(editor.hosts.len(), 2);
    assert_eq!(editor.sections.len(), 2);
    for pair in editor.hosts.windows(2) {
        assert!(pair[0].rect.bottom() <= pair[1].rect.y);
    }
    let last = editor.hosts.last().expect("a host line");
    assert!(last.rect.bottom() <= form.rect.y);
}

#[test]
fn a_click_on_a_listed_server_names_its_place_in_the_list() {
    let frame = listing(&["a", "b", "c"]);
    for (index, line) in editor_of(&frame).hosts.iter().enumerate() {
        assert_eq!(
            frame.hit(line.rect.center(), false),
            Some(Hit::EditHost(index))
        );
    }
}

#[test]
fn without_servers_the_window_holds_only_the_form() {
    let frame = listing(&[]);
    let editor = editor_of(&frame);
    assert!(editor.hosts.is_empty() && editor.sections.is_empty());
    assert!(editor.form.is_some());
}

#[test]
fn a_long_list_is_cut_to_what_the_window_can_show() {
    let names: Vec<String> = (0..200).map(|index| format!("host{index}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let frame = listing(&names);
    let editor = editor_of(&frame);
    let popup = editor.popup.as_ref().expect("a popup").rect;
    assert!(!editor.hosts.is_empty() && editor.hosts.len() < names.len());
    assert!(popup.bottom() <= frame.bounds.bottom());
    assert!(popup.y >= frame.title_bar.rect.bottom());
}
