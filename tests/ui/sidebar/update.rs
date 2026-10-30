use trame::geometry::Rect;
use trame::render::Scene;
use trame::text::TextSystem;
use trame::theme::{Appearance, Swatch, Theme};
use trame::ui::chrome::{draw_chrome, draw_tooltip};
use trame::ui::{FrameLayout, Hit, UiMetrics};
use trame::workspace::Workspace;

use super::footer::closed_sidebar;
use crate::paint::paint_with;
use crate::{input, text_system, workspace};

const LABEL: &str = "Update to 9.8.7";

fn offered(text: &mut TextSystem, workspace: &Workspace, update: Option<&str>) -> FrameLayout {
    let mut input = input(workspace, UiMetrics::new(1.0));
    input.update = update;
    FrameLayout::compute(text, &input)
}

#[test]
fn a_new_release_adds_a_button_above_the_settings() {
    let mut text = text_system();
    let layout = offered(&mut text, &workspace(), Some(LABEL));
    let footer = &layout.footer;
    let update = footer.update.as_ref().expect("the update button is shown");
    assert!(update.rect.bottom() < footer.settings.rect.y);
    assert!(footer.rect.contains(update.rect.center()));
    assert_eq!(update.rect.x, footer.settings.rect.x);
    assert_eq!(update.rect.width, footer.settings.rect.width);
    assert_eq!(layout.hit(update.rect.center(), false), Some(Hit::Update));
    let sidebar = layout.sidebar.as_ref().expect("the sidebar");
    assert!(sidebar.rect.bottom() <= footer.rect.y);
}

#[test]
fn the_button_shows_the_label_it_is_given() {
    let mut text = text_system();
    let layout = offered(&mut text, &workspace(), Some(LABEL));
    let update = layout.footer.update.as_ref().expect("the update button");
    let style = UiMetrics::new(1.0).label();
    let expected = text.layout(LABEL, style);
    assert_eq!(update.label.width, expected.width);
    assert!(update.label_x + update.label.width <= update.rect.right());
}

#[test]
fn without_a_new_release_the_footer_keeps_its_height() {
    let mut text = text_system();
    let plain = offered(&mut text, &workspace(), None);
    let offer = offered(&mut text, &workspace(), Some(LABEL));
    assert!(plain.footer.update.is_none());
    let button = offer.footer.settings.rect.height;
    let grown = offer.footer.rect.height - plain.footer.rect.height;
    assert!(grown > button && grown < 2.0 * button);
    assert_eq!(plain.footer.settings.rect, offer.footer.settings.rect);
}

#[test]
fn the_rail_keeps_only_the_icon_of_the_update_button() {
    let mut text = text_system();
    let layout = offered(&mut text, &closed_sidebar(), Some(LABEL));
    let update = layout.footer.update.as_ref().expect("the update button");
    assert_eq!(update.rect, update.icon);
    assert_eq!(layout.hit(update.rect.center(), false), Some(Hit::Update));
    let rail = layout.rail.as_ref().expect("the rail");
    assert!(rail.rect.bottom() <= layout.footer.rect.y);
}

fn inks(scene: &Scene, area: Rect, button: Rect) -> Vec<[f32; 4]> {
    let backdrop = [button.x, button.y, button.width, button.height];
    let quads = scene.quads().iter().filter(|quad| {
        let [x, y, width, height] = quad.rect;
        area.contains(Rect::new(x, y, width, height).center()) && quad.rect != backdrop
    });
    quads.map(|quad| quad.color).collect()
}

#[test]
fn the_update_button_is_inked_like_the_settings_gear() {
    let mut text = text_system();
    let layout = offered(&mut text, &workspace(), Some(LABEL));
    let footer = &layout.footer;
    let update = footer.update.as_ref().expect("the update button");
    let mut drawn = Vec::new();
    for appearance in [Appearance::Dark, Appearance::Light] {
        let theme = Theme::new(appearance);
        let scene = paint_with(&mut text, &layout, &theme, None, draw_chrome);
        let orange = theme.swatch(Swatch::Orange).to_array();
        assert!(!scene.quads().iter().any(|quad| quad.color == orange));
        let icon = inks(&scene, update.icon, update.rect);
        let gear = inks(&scene, footer.settings.icon, footer.settings.rect);
        let ink = theme.muted_text(0.0).to_array();
        assert!(!icon.is_empty() && icon.iter().all(|color| *color == ink));
        assert!(!gear.is_empty() && gear.iter().all(|color| *color == ink));
        drawn.push(ink);
    }
    assert_ne!(drawn[0], drawn[1]);
}

#[test]
fn hovering_the_rail_icon_names_the_version_beside_it() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let layout = offered(&mut text, &closed_sidebar(), Some(LABEL));
    let update = layout.footer.update.as_ref().expect("the update button");
    let rail = layout.rail.as_ref().expect("the rail");
    let scene = paint_with(&mut text, &layout, &theme, Some(Hit::Update), draw_tooltip);
    let bubble = scene
        .quads()
        .iter()
        .find(|quad| quad.color == theme.chrome.to_array())
        .expect("a bubble is drawn");
    let [x, y, width, height] = bubble.rect;
    let middle = update.rect.center().y;
    assert!(x >= rail.rect.right());
    assert!(y <= middle && y + height >= middle);
    assert!(width >= update.label.width);
}
