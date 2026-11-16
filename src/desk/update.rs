use crate::host::pending::Offer;
use crate::host::quit::{Quit, Restart};

use super::{Desk, Effect, Effects};

#[derive(Default)]
pub struct UpdateState {
    pub offer: Option<Offer>,
    pub failed: bool,
    pub restarting: bool,
}

impl Desk {
    pub fn offer_update(&mut self, offer: Offer) -> Effects {
        self.update.offer = Some(offer);
        Effect::Changed.into()
    }

    pub fn restart_for_update(&mut self) {
        if self.update.offer.is_some() {
            self.quit_with(Quit::Restart(Restart::Update));
        }
    }

    pub fn update_failed(&mut self) -> Effects {
        self.update.failed = true;
        Effect::Changed.into()
    }

    pub fn mark_restarting(&mut self) {
        self.update.restarting = true;
        self.saver.stop();
    }
}
