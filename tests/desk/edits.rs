use trame::desk::{Desk, Edit, Effect, Placement};
use trame::theme::{Emblem, Letters, Swatch};
use trame::workspace::{EditTarget, InstanceId};

use crate::sandbox::Sandbox;
use crate::tabs::opened;

fn targets() -> (Desk, EditTarget, EditTarget) {
    let (desk, tab) = opened();
    let instance = desk.workspace.active_instance().expect("an instance").id;
    (desk, EditTarget::Tab(tab), EditTarget::Instance(instance))
}

fn instance_of(target: EditTarget) -> InstanceId {
    match target {
        EditTarget::Instance(id) => id,
        EditTarget::Tab(_) => panic!("not an instance"),
    }
}

fn name(text: &str, original: &str) -> Edit {
    Edit::Name {
        text: text.to_owned(),
        original: original.to_owned(),
    }
}

#[test]
fn an_unchanged_name_changes_nothing() {
    let (mut desk, tab, instance) = targets();
    let before = desk.workspace.instance(instance_of(instance)).cloned();
    assert!(desk.commit(tab, name("same", "same")).is_empty());
    assert!(desk.commit(instance, name("same", "same")).is_empty());
    let EditTarget::Tab(id) = tab else { return };
    assert_eq!(desk.workspace.tab_name(id), None);
    let after = desk.workspace.instance(instance_of(instance)).cloned();
    assert_eq!(before.map(|item| item.name), after.map(|item| item.name));
}

#[test]
fn names_go_to_tabs_and_instances() {
    let (mut desk, tab, instance) = targets();
    assert!(desk.commit(tab, name("logs", "")).is_empty());
    assert!(desk.commit(instance, name("work", "")).is_empty());
    let EditTarget::Tab(id) = tab else { return };
    assert_eq!(desk.workspace.tab_name(id), Some("logs"));
    let shown = desk.workspace.instance(instance_of(instance));
    assert_eq!(shown.map(|item| item.name.as_str()), Some("work"));
}

#[test]
fn colors_and_fills_are_set_and_cleared() {
    let (mut desk, tab, instance) = targets();
    for target in [tab, instance] {
        assert!(
            desk.commit(target, Edit::Color(Some(Swatch::Red)))
                .is_empty()
        );
        assert_eq!(desk.color_of(target), Some(Swatch::Red));
        assert!(desk.commit(target, Edit::Color(None)).is_empty());
        assert_eq!(desk.color_of(target), None);
        assert!(desk.commit(target, Edit::Fill(true)).is_empty());
        assert!(desk.fill_of(target));
        assert!(desk.commit(target, Edit::Fill(false)).is_empty());
        assert!(!desk.fill_of(target));
    }
}

#[test]
fn emblems_only_go_to_instances() {
    let (mut desk, tab, instance) = targets();
    let letters = Letters::parse("AB").expect("letters");
    let emblem = Emblem::Letters(letters);
    assert!(desk.commit(tab, Edit::Emblem(emblem)).is_empty());
    assert_eq!(desk.emblem_of(tab), None);
    assert!(desk.commit(instance, Edit::Emblem(emblem)).is_empty());
    assert_eq!(desk.emblem_of(instance), Some(emblem));
}

#[test]
fn a_new_custom_color_is_kept_and_painted() {
    let (mut desk, tab, _) = targets();
    let color = Swatch::custom(0x12_34_56);
    let edit = Edit::CustomColor { slot: None, color };
    assert_eq!(desk.commit(tab, edit).as_slice(), [Effect::Changed]);
    assert_eq!(desk.preferences.custom_colors, [color]);
    assert_eq!(desk.color_of(tab), Some(color));
    assert_eq!(desk.forget_custom_color(0).as_slice(), [Effect::Changed]);
    assert!(desk.preferences.custom_colors.is_empty());
}

#[test]
fn a_changed_custom_color_is_replaced_everywhere() {
    let (mut desk, tab, first) = targets();
    let old = Swatch::custom(0x11_11_11);
    let new = Swatch::custom(0x22_22_22);
    let _ = desk.commit(
        tab,
        Edit::CustomColor {
            slot: None,
            color: old,
        },
    );
    let _ = desk.open(Placement::Instance);
    let second = desk.workspace.active_instance().expect("an instance").id;
    let second = EditTarget::Instance(second);
    let _ = desk.commit(second, Edit::Color(Some(old)));
    let edit = Edit::CustomColor {
        slot: Some(0),
        color: new,
    };
    let _ = desk.commit(first, edit);
    assert_eq!(desk.preferences.custom_colors, [new]);
    for target in [tab, first, second] {
        assert_eq!(desk.color_of(target), Some(new));
    }
}

#[test]
fn a_picked_icon_becomes_the_emblem() {
    let (mut desk, _, instance) = targets();
    let effects = desk.icon_picked(instance_of(instance), 42);
    assert_eq!(effects.as_slice(), [Effect::Changed]);
    assert_eq!(desk.emblem_of(instance), Some(Emblem::Image(42)));
}

#[test]
fn a_folder_edit_relaunches_the_tab() {
    let sandbox = Sandbox::new("desk-edit-folder");
    let (mut desk, tab, _) = targets();
    let effects = desk.commit(tab, Edit::Folder(sandbox.root.clone()));
    assert!(!effects.is_empty());
}
