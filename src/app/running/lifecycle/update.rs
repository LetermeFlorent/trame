use std::io;

use winit::event_loop::ActiveEventLoop;

use trame::locale::Strings;

use crate::app::running::Running;
use crate::app::update;

const VERSION_SLOT: &str = "{version}";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub version: String,
}

impl Offer {
    pub fn label(&self, strings: &Strings) -> String {
        format!("{} {}", strings.update_to, self.version)
    }

    pub fn question(&self, strings: &Strings) -> String {
        strings.update_question.replace(VERSION_SLOT, &self.version)
    }
}

impl Running {
    pub fn offer_update(&mut self, offer: Offer) {
        self.update_offer = Some(offer);
        self.changed();
    }

    pub(in crate::app::running) fn restart_for_update(&mut self, event_loop: &ActiveEventLoop) {
        if self.update_offer.is_none() {
            return;
        }
        self.restart_with(
            event_loop,
            update::restart,
            "the update restart did not start",
        );
    }

    pub(in crate::app::running) fn restart_if_card_lost(&mut self, event_loop: &ActiveEventLoop) {
        if self.restarting || !self.graphics.is_lost() {
            return;
        }
        self.restart_with(
            event_loop,
            update::start_again,
            "trame could not start again",
        );
    }

    fn restart_with(
        &mut self,
        event_loop: &ActiveEventLoop,
        start: impl FnOnce() -> io::Result<()>,
        failure: &str,
    ) {
        self.save_for_restart();
        match start() {
            Ok(()) => {
                self.restarting = true;
                event_loop.exit();
            }
            Err(error) => eprintln!("trame: {failure}: {error}"),
        }
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
        assert_eq!(offer().label(strings), "Mettre à jour en 9.8.7");
        assert_eq!(offer().question(strings), "Mettre à jour en 9.8.7 ?");
    }
}
