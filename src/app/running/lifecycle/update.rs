use std::io;

use winit::event_loop::ActiveEventLoop;

use trame::locale::{Strings, VERSION_SLOT};
use trame::log_error;

use crate::app::running::Running;
use crate::app::update;

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

    pub(in crate::app::running) fn restart_for_update(&mut self, event_loop: &ActiveEventLoop) {
        if self.update.offer.is_none() {
            return;
        }
        if let Err(error) = self.restart_with(event_loop, update::restart) {
            log_error!("the update restart did not start: {error}");
            self.update.failed = true;
            self.changed();
        }
    }

    pub(in crate::app::running) fn restart_if_card_lost(&mut self, event_loop: &ActiveEventLoop) {
        if self.update.restarting || !self.graphics.is_lost() {
            return;
        }
        if let Err(error) = self.restart_with(event_loop, update::start_again) {
            log_error!("trame could not start again: {error}");
        }
    }

    pub fn restart_with(
        &mut self,
        event_loop: &ActiveEventLoop,
        start: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        self.save_for_restart();
        self.saver.hand_over(start)?;
        self.update.restarting = true;
        event_loop.exit();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trame::locale::Language;

    fn offer() -> Offer {
        Offer {
            version: "9.8.7".to_owned(),
        }
    }

    #[test]
    fn an_offer_asks_to_update() {
        let strings = Language::English.strings();
        assert_eq!(offer().label(strings), "Update to 9.8.7");
        assert_eq!(offer().question(strings), "Update to 9.8.7?");
    }

    #[test]
    fn the_french_question_names_the_version() {
        let strings = Language::French.strings();
        assert_eq!(offer().label(strings), "Installer la 9.8.7");
        assert_eq!(offer().question(strings), "Mettre à jour en 9.8.7 ?");
    }
}
