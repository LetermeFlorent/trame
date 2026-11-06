use trame::locale::Language;
use trame::ui::Hit;

use crate::{frame, text_system, workspace};

#[test]
fn a_waiting_ssh_tab_offers_a_disconnect_button_over_its_terminal() {
    let mut text = text_system();
    let workspace = workspace();
    let mut layout = frame(&mut text, &workspace);
    let strings = Language::English.strings();
    let tab = layout.panes[0].active.expect("a shown tab");
    layout.mark_offline(&mut text, strings, |id| id == tab);
    let card = layout.panes[0]
        .offline
        .clone()
        .expect("the waiting tab gets a card");
    let body = layout.panes[0].body;
    assert_eq!(card.tab, tab);
    assert!(body.contains(card.button.center()) && body.contains(card.line.center()));
    assert!(card.line.bottom() <= card.button.y);
    let hit = layout.hit(card.button.center(), false);
    assert_eq!(hit, Some(Hit::Disconnect(tab)));
    layout.mark_offline(&mut text, strings, |_| false);
    assert!(layout.panes[0].offline.is_none());
}
