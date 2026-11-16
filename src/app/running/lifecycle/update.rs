use trame::host::pending::{Offer, ShowsOffer};
use trame::host::quit::{Quit, Restart};

use crate::app::running::Running;

impl Running {
    pub fn offer_update(&mut self, offer: Offer) {
        let effects = self.desk.offer_update(offer);
        self.apply(effects);
    }

    pub(in crate::app::running) fn restart_if_card_lost(&mut self) {
        if !self.desk.update.restarting && self.graphics.is_lost() {
            self.desk.quit_with(Quit::Restart(Restart::Again));
        }
    }

    pub fn update_failed(&mut self) {
        let effects = self.desk.update_failed();
        self.apply(effects);
    }

    pub fn mark_restarting(&mut self) {
        self.desk.mark_restarting();
    }
}

impl ShowsOffer for Running {
    fn show_offer(&mut self, offer: Offer) {
        self.offer_update(offer);
    }
}
