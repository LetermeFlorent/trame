use trame::host::pending::{Offer, ShowsOffer};
use trame::host::quit::{Quit, Restart};

use crate::app::running::Running;

#[derive(Default)]
pub(in crate::app::running) struct UpdateState {
    pub(in crate::app::running) offer: Option<Offer>,
    pub(in crate::app::running) failed: bool,
    pub(in crate::app::running) restarting: bool,
}

impl Running {
    pub fn offer_update(&mut self, offer: Offer) {
        self.update.offer = Some(offer);
        self.changed();
    }

    pub(in crate::app::running) fn restart_for_update(&mut self) {
        if self.update.offer.is_some() {
            self.quit_with(Quit::Restart(Restart::Update));
        }
    }

    pub(in crate::app::running) fn restart_if_card_lost(&mut self) {
        if !self.update.restarting && self.graphics.is_lost() {
            self.quit_with(Quit::Restart(Restart::Again));
        }
    }

    pub fn update_failed(&mut self) {
        self.update.failed = true;
        self.changed();
    }

    pub fn mark_restarting(&mut self) {
        self.update.restarting = true;
        self.saver.stop();
    }
}

impl ShowsOffer for Running {
    fn show_offer(&mut self, offer: Offer) {
        self.offer_update(offer);
    }
}
