use trame::geometry::Point;
use trame::locale::Language;
use trame::preferences::Preferences;
use trame::ui::Hit;

use super::super::footer::closed_sidebar;
use super::{english, general_request, panel};
use crate::{frame, text_system};

#[test]
fn the_window_opens_in_the_middle_over_everything_below_the_title_bar() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    assert_eq!(settings.popup.backdrop.y, layout.title_bar.rect.bottom());
    assert_eq!(settings.popup.backdrop.bottom(), layout.bounds.bottom());
    assert_eq!(settings.popup.backdrop.width, layout.bounds.width);
    let (panel, backdrop) = (
        settings.popup.rect.center(),
        settings.popup.backdrop.center(),
    );
    assert!((panel.x - backdrop.x).abs() <= 1.0);
    assert!((panel.y - backdrop.y).abs() <= 1.0);
}

#[test]
fn a_click_beside_the_window_reaches_nothing_but_the_title_bar_still_works() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    let row = layout.sidebar.as_ref().expect("the sidebar").rows[0].rect;
    let content = Point::new(
        settings.popup.rect.right() + 10.0,
        settings.popup.rect.center().y,
    );
    for point in [row.center(), content, layout.footer.settings.rect.center()] {
        assert_eq!(layout.hit(point, false), Some(Hit::Backdrop));
    }
    if let Some(controls) = layout.title_bar.controls {
        assert_eq!(layout.hit(controls.close.center(), false), Some(Hit::Close));
    }
}

#[test]
fn a_collapsed_sidebar_changes_nothing_to_the_window() {
    let mut text = text_system();
    let mut layout = frame(&mut text, &closed_sidebar());
    let preferences = Preferences::default();
    let request = general_request(&preferences, Language::English.strings());
    layout.place_settings(&mut text, &request);
    let open = english(&Preferences::default());
    assert_eq!(panel(&layout).popup.rect, panel(&open).popup.rect);
}
