use trame::input::TextField;
use trame::locale::Language;
use trame::render::Scene;
use trame::text::TextSystem;
use trame::theme::{Appearance, Theme};
use trame::ui::chrome::{ChromeInput, draw_tooltip};
use trame::ui::frame::{EditAnchor, EditKind, EditTarget, EditorRequest};
use trame::ui::{FrameLayout, FrameView, Hit, Motion};
use trame::workspace::Workspace;

use crate::{frame, text_system, workspace};

fn collapsed() -> Workspace {
    let mut workspace = workspace();
    workspace.rename_instance(workspace.instances[1].id, "backend");
    workspace.sidebar_open = false;
    workspace
}

fn paint(text: &mut TextSystem, frame: &FrameLayout, theme: &Theme, hover: Option<Hit>) -> Scene {
    let view = FrameView {
        hover,
        pressed: None,
        window_focused: true,
    };
    let motion = Motion::default();
    let input = ChromeInput {
        theme,
        frame,
        view: &view,
        motion: &motion,
    };
    let mut scene = Scene::new(frame.bounds.width, frame.bounds.height);
    draw_tooltip(&mut scene, text, &input);
    scene
}

#[test]
fn hovering_a_rail_icon_shows_the_instance_name_beside_the_rail() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let workspace = collapsed();
    let layout = frame(&mut text, &workspace);
    let rail = layout.rail.as_ref().expect("the rail");
    let slot = &rail.slots[1];
    let scene = paint(&mut text, &layout, &theme, Some(Hit::RailInstance(slot.id)));
    let bubble = scene
        .quads()
        .iter()
        .find(|quad| quad.color == theme.chrome.to_array())
        .expect("a bubble is drawn");
    let [x, y, width, height] = bubble.rect;
    assert!(x >= rail.rect.right());
    assert!(y <= slot.rect.center().y && y + height >= slot.rect.center().y);
    let name = text.layout("backend", layout.metrics.label());
    assert!(width >= name.width);
    assert!(scene.quads().len() > 3);
}

#[test]
fn nothing_is_shown_without_a_hovered_rail_icon() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &collapsed());
    assert!(paint(&mut text, &layout, &theme, None).quads().is_empty());
    let open = frame(&mut text, &workspace());
    let id = workspace().instances[0].id;
    assert!(
        paint(&mut text, &open, &theme, Some(Hit::RailInstance(id)))
            .quads()
            .is_empty()
    );
}

#[test]
fn the_name_hides_while_the_editor_is_open() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let workspace = collapsed();
    let mut layout = frame(&mut text, &workspace);
    let id = workspace.instances[1].id;
    let field = TextField::new("backend");
    let request = EditorRequest {
        target: EditTarget::Instance(id),
        anchor: EditAnchor::Rail,
        kind: EditKind::Name(&field),
        strings: Language::English.strings(),
    };
    layout.place_editor(&mut text, &request);
    assert!(layout.editor.is_some());
    assert!(
        paint(&mut text, &layout, &theme, Some(Hit::RailInstance(id)))
            .quads()
            .is_empty()
    );
}

#[test]
fn the_rail_gear_names_the_settings_beside_it() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = frame(&mut text, &collapsed());
    let rail = layout.rail.as_ref().expect("the rail");
    let gear = layout.footer.settings;
    let scene = paint(&mut text, &layout, &theme, Some(Hit::Settings));
    let bubble = scene
        .quads()
        .iter()
        .find(|quad| quad.color == theme.chrome.to_array())
        .expect("a bubble is drawn");
    let [x, y, width, height] = bubble.rect;
    assert!(x >= rail.rect.right());
    assert!(y <= gear.center().y && y + height >= gear.center().y);
    assert!(width >= layout.footer.label.width);
    let open = frame(&mut text, &workspace());
    assert!(
        paint(&mut text, &open, &theme, Some(Hit::Settings))
            .quads()
            .is_empty()
    );
}
