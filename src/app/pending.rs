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
    pub fn send(&mut self, offer: Offer, window: Option<&mut impl ShowsOffer>) {
        match window {
            Some(window) => window.show_offer(offer),
            None => self.0 = Some(offer),
        }
    }

    pub fn opened(&mut self, window: &mut impl ShowsOffer) {
        if let Some(offer) = self.0.take() {
            window.show_offer(offer);
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
    fn an_offer_sent_before_the_window_shows_once_it_opens() {
        let mut pending = PendingOffer::default();
        pending.send(offer("9.0.0"), None::<&mut Vec<Offer>>);
        let mut window = Vec::new();
        pending.opened(&mut window);
        pending.opened(&mut window);
        assert_eq!(window, [offer("9.0.0")]);
    }

    #[test]
    fn the_latest_early_offer_is_the_one_kept() {
        let mut pending = PendingOffer::default();
        pending.send(offer("9.0.0"), None::<&mut Vec<Offer>>);
        pending.send(offer("9.1.0"), None::<&mut Vec<Offer>>);
        let mut window = Vec::new();
        pending.opened(&mut window);
        assert_eq!(window, [offer("9.1.0")]);
    }

    #[test]
    fn an_offer_sent_to_an_open_window_shows_at_once() {
        let mut pending = PendingOffer::default();
        let mut window = Vec::new();
        pending.send(offer("9.0.0"), Some(&mut window));
        assert_eq!(window, [offer("9.0.0")]);
        pending.opened(&mut window);
        assert_eq!(window.len(), 1);
    }
}
