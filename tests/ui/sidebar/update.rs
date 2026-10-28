use trame::locale::Language;
use trame::text::TextSystem;
use trame::theme::{Appearance, Swatch, Theme};
use trame::ui::chrome::{draw_chrome, draw_tooltip};
use trame::ui::{FrameLayout, Hit, UiMetrics};
use trame::workspace::Workspace;

use super::footer::closed_sidebar;
use crate::paint::paint_with;
use crate::{input, text_system, workspace};

const LABEL: &str = "Restart into 9.8.7";
const VERSION: &str = "0.3.20";

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

#[test]
fn the_update_button_is_painted_orange() {
    let mut text = text_system();
    let theme = Theme::new(Appearance::Dark);
    let orange = theme.swatch(Swatch::Orange).to_array();
    let layout = offered(&mut text, &workspace(), Some(LABEL));
    let update = layout.footer.update.as_ref().expect("the update button");
    let scene = paint_with(&mut text, &layout, &theme, None, draw_chrome);
    assert!(
        scene.quads().iter().any(|quad| {
            quad.color == orange && quad.rect[..2] == [update.rect.x, update.rect.y]
        })
    );
    let plain = offered(&mut text, &workspace(), None);
    let scene = paint_with(&mut text, &plain, &theme, None, draw_chrome);
    assert!(!scene.quads().iter().any(|quad| quad.color == orange));
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

#[test]
fn every_language_shows_the_whole_label_in_the_narrowest_sidebar() {
    let mut text = text_system();
    let mut workspace = workspace();
    workspace.sidebar_width = Some(UiMetrics::SIDEBAR_MIN_WIDTH);
    for language in [Language::French, Language::English] {
        let strings = language.strings();
        for lead in [strings.update_to, strings.restart_into] {
            let label = format!("{lead} {VERSION}");
            let layout = offered(&mut text, &workspace, Some(&label));
            let update = layout.footer.update.as_ref().expect("the update button");
            let end = update.label_x + update.label.width;
            assert!(end <= update.rect.right(), "{label} ends at {end}");
        }
    }
}
