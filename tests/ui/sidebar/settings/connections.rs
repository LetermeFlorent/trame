use trame::geometry::Point;
use trame::locale::Language;
use trame::preferences::{ConnectForm, Preferences, SshHost};
use trame::ui::frame::{
    ConnectionsPage, FormAction, HostList, HostsView, ListControl, SettingControl, SettingsRequest,
    SettingsTab,
};
use trame::ui::{FrameLayout, Hit};

use super::{general_request, panel, placed};
use crate::text_system;

pub fn hosts(count: usize) -> Vec<SshHost> {
    (0..count)
        .map(|index| SshHost::parse(&format!("me@box{index}")).expect("a valid connection"))
        .collect()
}

pub fn opened(hosts: &[SshHost], first: usize, form: Option<&ConnectForm>) -> FrameLayout {
    let preferences = Preferences::default();
    let view = HostsView {
        hosts,
        first,
        form,
        changing: false,
    };
    let request = SettingsRequest {
        tab: SettingsTab::Connections,
        hosts: Some(view),
        ..general_request(&preferences, Language::English.strings())
    };
    placed(&mut text_system(), &request)
}

fn page(layout: &FrameLayout) -> &ConnectionsPage {
    panel(layout).connections().expect("the connections page")
}

fn list(layout: &FrameLayout) -> &HostList {
    page(layout).list.as_ref().expect("a list")
}

#[test]
fn without_saved_connections_the_tab_explains_how_to_add_one() {
    let layout = opened(&[], 0, None);
    let page = page(&layout);
    assert!(page.list.is_none() && page.form.is_none());
    let hint = page.hint.as_ref().expect("a hint");
    assert_eq!(
        layout.hit(hint.rect.center(), false),
        Some(Hit::SettingsPanel)
    );
    let add = &page.header.chips[0];
    assert_eq!(add.control, SettingControl::AddHost);
    assert_eq!(
        layout.hit(add.rect.center(), false),
        Some(Hit::Setting(SettingControl::AddHost))
    );
}

#[test]
fn each_saved_connection_has_a_pencil_and_a_cross() {
    let saved = hosts(3);
    let layout = opened(&saved, 0, None);
    let list = list(&layout);
    assert!(list.bar.is_none() && !list.pickable);
    assert_eq!(list.rows.len(), 3);
    for (index, row) in list.rows.iter().enumerate() {
        let forget = row.forget.expect("a cross");
        assert!(row.change.right() <= forget.x);
        assert_eq!(
            layout.hit(row.change.center(), false),
            Some(Hit::List(ListControl::Change(index)))
        );
        assert_eq!(
            layout.hit(forget.center(), false),
            Some(Hit::List(ListControl::Forget(index)))
        );
    }
    for pair in list.rows.windows(2) {
        assert!(pair[0].line.rect.bottom() <= pair[1].line.rect.y);
    }
}

#[test]
fn a_long_list_shows_what_fits_from_the_first_row_asked() {
    let saved = hosts(60);
    let layout = opened(&saved, 5, None);
    let list = list(&layout);
    let bar = list.bar.expect("a scroll bar");
    assert_eq!(list.rows[0].index, 5);
    assert_eq!(bar.shown, list.rows.len());
    assert_eq!(bar.last_first, saved.len() - bar.shown);
    assert!(list.rows.len() < saved.len());
    let popup = panel(&layout).popup.rect;
    assert!(list.rect.bottom() <= popup.bottom());
    assert!(bar.track.contains(bar.thumb.center()));
    assert_eq!(
        layout.hit(bar.thumb.center(), false),
        Some(Hit::List(ListControl::Thumb))
    );
    let below = Point::new(bar.track.center().x, bar.track.bottom() - 1.0);
    assert_eq!(
        layout.hit(below, false),
        Some(Hit::List(ListControl::Track))
    );
}

#[test]
fn the_scroll_bar_moves_by_rows_and_by_pages() {
    let saved = hosts(60);
    let layout = opened(&saved, 0, None);
    let bar = list(&layout).bar.expect("a scroll bar");
    assert_eq!(bar.scrolled(-3), 0);
    assert_eq!(bar.scrolled(3), 3);
    assert_eq!(bar.scrolled(1000), bar.last_first);
    assert_eq!(bar.paged(bar.track.bottom()), bar.shown);
    assert_eq!(bar.first_at(bar.track.bottom()), bar.last_first);
    assert_eq!(bar.first_at(bar.track.y), 0);
}

#[test]
fn the_form_replaces_the_list_with_cancel_and_save() {
    let form = ConnectForm::new();
    let layout = opened(&hosts(2), 0, Some(&form));
    let page = page(&layout);
    assert!(page.list.is_none() && page.hint.is_none());
    let frame = page.form.as_ref().expect("a form");
    let actions: Vec<FormAction> = frame.buttons.iter().map(|button| button.action).collect();
    assert_eq!(actions, [FormAction::Cancel, FormAction::Save]);
    for button in &frame.buttons {
        assert_eq!(
            layout.hit(button.rect.center(), false),
            Some(Hit::FormButton(button.action))
        );
    }
    for row in &frame.rows {
        assert_eq!(
            layout.hit(row.input.rect.center(), false),
            Some(Hit::FormField(row.field))
        );
    }
    assert!(frame.rect.bottom() <= panel(&layout).popup.rect.bottom());
}
