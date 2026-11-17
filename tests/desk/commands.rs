use trame::desk::{Desk, Effect, Effects};
use trame::geometry::Rect;
use trame::input::Command;
use trame::workspace::{Edge, PaneLayout};

use crate::tabs::opened;

fn run(desk: &mut Desk, command: Command) -> Option<Effects> {
    let panes = PaneLayout {
        panes: Vec::new(),
        dividers: Vec::new(),
    };
    desk.command(false, command, &panes)
}

fn framed(effects: Option<Effects>) -> Vec<Effect> {
    let effects = effects.expect("a desk command").as_slice().to_vec();
    assert_eq!(effects.first(), Some(&Effect::Animate));
    assert_eq!(effects.last(), Some(&Effect::Changed));
    effects
}

#[test]
fn view_commands_stay_with_the_front_end() {
    let (mut desk, _) = opened();
    for command in [
        Command::Copy,
        Command::Paste,
        Command::ScrollPageUp,
        Command::ScrollPageDown,
        Command::ScrollTop,
        Command::ScrollBottom,
    ] {
        assert!(run(&mut desk, command).is_none());
    }
}

#[test]
fn a_new_tab_is_framed_by_animate_and_changed() {
    let (mut desk, _) = opened();
    let effects = framed(run(&mut desk, Command::NewTab));
    assert!(effects.contains(&Effect::StartQueued));
    assert_eq!(desk.workspace.tabs().len(), 2);
}

#[test]
fn tabs_cycle_and_wrap_within_their_pane() {
    let (mut desk, first) = opened();
    framed(run(&mut desk, Command::NewTab));
    let second = desk.workspace.active_tab();
    framed(run(&mut desk, Command::NextTab));
    assert_eq!(desk.workspace.active_tab(), Some(first));
    framed(run(&mut desk, Command::PreviousTab));
    assert_eq!(desk.workspace.active_tab(), second);
}

#[test]
fn instances_cycle_only_when_there_are_several() {
    let (mut desk, _) = opened();
    framed(run(&mut desk, Command::NextInstance));
    assert_eq!(desk.workspace.active, 0);
    framed(run(&mut desk, Command::NewInstance));
    assert_eq!(desk.workspace.active, 1);
    framed(run(&mut desk, Command::NextInstance));
    assert_eq!(desk.workspace.active, 0);
    framed(run(&mut desk, Command::PreviousInstance));
    assert_eq!(desk.workspace.active, 1);
}

#[test]
fn the_sidebar_toggles_to_its_next_mode() {
    let (mut desk, _) = opened();
    let next = desk.workspace.sidebar.next();
    framed(run(&mut desk, Command::ToggleSidebar));
    assert_eq!(desk.workspace.sidebar, next);
}

#[test]
fn a_bigger_font_asks_for_a_rescale() {
    let (mut desk, _) = opened();
    let before = desk.preferences.font_size;
    let effects = framed(run(&mut desk, Command::FontBigger));
    assert!(effects.contains(&Effect::Rescale));
    assert!(desk.preferences.font_size > before);
}

#[test]
fn focus_moves_to_the_pane_on_the_right() {
    let (mut desk, _) = opened();
    let left = desk
        .workspace
        .active_instance()
        .expect("an instance")
        .active_pane;
    framed(run(&mut desk, Command::SplitRight));
    let instance = desk.workspace.active_instance_mut().expect("an instance");
    let right = instance.active_pane;
    assert_ne!(left, right);
    instance.focus(left);
    let panes = PaneLayout {
        panes: vec![
            (left, Rect::new(0.0, 0.0, 100.0, 100.0)),
            (right, Rect::new(100.0, 0.0, 100.0, 100.0)),
        ],
        dividers: Vec::new(),
    };
    framed(desk.command(false, Command::FocusPane(Edge::Right), &panes));
    let instance = desk.workspace.active_instance().expect("an instance");
    assert_eq!(instance.active_pane, right);
}
