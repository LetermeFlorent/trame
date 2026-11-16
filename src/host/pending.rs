use crate::locale::{Strings, VERSION_SLOT};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub version: String,
}

impl Offer {
    pub fn label(&self, strings: &Strings) -> String {
        strings.update_to.replace(VERSION_SLOT, &self.version)
    }

    pub fn question(&self, strings: &Strings) -> String {
        strings.update_question.replace(VERSION_SLOT, &self.version)
    }
}

pub trait ShowsOffer {
    fn show_offer(&mut self, offer: Offer);
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
