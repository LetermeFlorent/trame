use super::running::{Offer, Running};

pub trait ShowsOffer {
    fn show_offer(&mut self, offer: Offer);
}

impl ShowsOffer for Running {
    fn show_offer(&mut self, offer: Offer) {
        self.offer_update(offer);
    }
}

#[derive(Default)]
pub struct PendingOffer(Option<Offer>);

impl PendingOffer {
    pub fn send(&mut self, offer: Offer, windows: &mut [impl ShowsOffer]) {
        for window in windows.iter_mut() {
            window.show_offer(offer.clone());
        }
        self.0 = Some(offer);
    }

    pub fn opened(&self, window: &mut impl ShowsOffer) {
        if let Some(offer) = &self.0 {
            window.show_offer(offer.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ShowsOffer for Vec<Offer> {
        fn show_offer(&mut self, offer: Offer) {
            self.push(offer);
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
        pending.send(offer("9.0.0"), &mut [] as &mut [Vec<Offer>]);
        let (mut first, mut second) = (Vec::new(), Vec::new());
        pending.opened(&mut first);
        pending.opened(&mut second);
        assert_eq!(first, [offer("9.0.0")]);
        assert_eq!(second, [offer("9.0.0")]);
    }

    #[test]
    fn the_latest_offer_is_the_one_kept() {
        let mut pending = PendingOffer::default();
        pending.send(offer("9.0.0"), &mut [] as &mut [Vec<Offer>]);
        pending.send(offer("9.1.0"), &mut [] as &mut [Vec<Offer>]);
        let mut window = Vec::new();
        pending.opened(&mut window);
        assert_eq!(window, [offer("9.1.0")]);
    }

    #[test]
    fn an_offer_shows_at_once_in_every_open_window() {
        let mut pending = PendingOffer::default();
        let mut windows = [Vec::new(), Vec::new()];
        pending.send(offer("9.0.0"), &mut windows);
        assert_eq!(windows, [[offer("9.0.0")], [offer("9.0.0")]]);
    }
}
