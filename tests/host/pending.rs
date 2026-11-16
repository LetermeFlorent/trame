use trame::host::pending::{Offer, PendingOffer, ShowsOffer};
use trame::locale::Language;

#[derive(Debug, Default, PartialEq)]
struct Window(Vec<Offer>);

impl ShowsOffer for Window {
    fn show_offer(&mut self, offer: Offer) {
        self.0.push(offer);
    }
}

fn offer(version: &str) -> Offer {
    Offer {
        version: version.to_owned(),
    }
}

#[test]
fn an_offer_sent_before_any_window_shows_in_each_window_that_opens() {
    let mut pending = PendingOffer::default();
    pending.send(offer("9.0.0"), &mut [] as &mut [Window]);
    let (mut first, mut second) = (Window::default(), Window::default());
    pending.opened(&mut first);
    pending.opened(&mut second);
    assert_eq!(first.0, [offer("9.0.0")]);
    assert_eq!(second.0, [offer("9.0.0")]);
}

#[test]
fn the_latest_offer_is_the_one_kept() {
    let mut pending = PendingOffer::default();
    pending.send(offer("9.0.0"), &mut [] as &mut [Window]);
    pending.send(offer("9.1.0"), &mut [] as &mut [Window]);
    let mut window = Window::default();
    pending.opened(&mut window);
    assert_eq!(window.0, [offer("9.1.0")]);
}

#[test]
fn an_offer_shows_at_once_in_every_open_window() {
    let mut pending = PendingOffer::default();
    let mut windows = [Window::default(), Window::default()];
    pending.send(offer("9.0.0"), &mut windows);
    assert_eq!(
        windows.map(|window| window.0),
        [[offer("9.0.0")], [offer("9.0.0")]]
    );
}

#[test]
fn an_offer_asks_to_update() {
    let strings = Language::English.strings();
    assert_eq!(offer("9.8.7").label(strings), "Update to 9.8.7");
    assert_eq!(offer("9.8.7").question(strings), "Update to 9.8.7?");
}

#[test]
fn the_french_question_names_the_version() {
    let strings = Language::French.strings();
    assert_eq!(offer("9.8.7").label(strings), "Installer la 9.8.7");
    assert_eq!(offer("9.8.7").question(strings), "Mettre à jour en 9.8.7 ?");
}
