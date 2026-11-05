use trame::locale::{Strings, VERSION_SLOT};

use super::quit::{Quit, Restart};
use crate::app::running::Running;

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
