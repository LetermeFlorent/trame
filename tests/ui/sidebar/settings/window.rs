use trame::geometry::Point;
use trame::locale::Language;
use trame::preferences::Preferences;
use trame::ui::Hit;
use trame::ui::frame::{FormPage, SettingsTab};

use super::super::footer::closed_sidebar;
use super::{FORM_TABS, english, general_request, on_tab, page, panel};
use crate::{frame, text_system};

const PANEL_PAD: f32 = 12.0;

fn lowest(page: &FormPage) -> f32 {
    let toggles = page.toggles.iter().map(|toggle| toggle.line.rect);
    let steppers = page.steppers.iter().map(|stepper| stepper.line.rect);
    let rows = page.rows.iter().map(|row| row.line.rect);
    toggles
        .chain(steppers)
        .chain(rows)
        .map(|rect| rect.bottom())
        .fold(0.0, f32::max)
}

#[test]
fn the_window_opens_centred_across_and_never_low_over_everything_below_the_title_bar() {
    let layout = english(&Preferences::default());
    let settings = panel(&layout);
    let (rect, backdrop) = (settings.popup.rect, settings.popup.backdrop);
    assert_eq!(backdrop.y, layout.title_bar.rect.bottom());
    assert_eq!(backdrop.bottom(), layout.bounds.bottom());
    assert_eq!(backdrop.width, layout.bounds.width);
    assert!((rect.center().x - backdrop.center().x).abs() <= 1.0);
    let (above, below) = (rect.y - backdrop.y, backdrop.bottom() - rect.bottom());
    assert!(above >= 0.0 && above <= below + 1.0, "{above} {below}");
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

#[test]
fn a_short_tab_ends_under_its_last_row_with_its_top_in_place() {
    let preferences = Preferences::default();
    let full = panel(&on_tab(&preferences, SettingsTab::Shortcuts))
        .popup
        .rect;
    for tab in FORM_TABS {
        let layout = on_tab(&preferences, tab);
        let popup = panel(&layout).popup.rect;
        assert_eq!(
            (popup.x, popup.y, popup.width),
            (full.x, full.y, full.width)
        );
        let below = popup.bottom() - lowest(page(&layout));
        assert!((below - PANEL_PAD).abs() <= 1.0, "{tab:?}: {below}");
    }
    let instances = panel(&on_tab(&preferences, SettingsTab::Instances))
        .popup
        .rect;
    assert_eq!(instances.y, full.y);
    assert!(instances.height < full.height);
}
