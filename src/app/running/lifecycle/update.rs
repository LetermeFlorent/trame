use std::io;
use std::path::PathBuf;

use winit::event_loop::ActiveEventLoop;

use trame::locale::Strings;

use crate::app::running::Running;
use crate::app::update;

const VERSION_SLOT: &str = "{version}";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub version: String,
    pub ready: Option<PathBuf>,
}

impl Offer {
    pub fn label(&self, strings: &Strings) -> String {
        let lead = match self.ready {
            Some(_) => strings.restart_into,
            None => strings.update_to,
        };
        format!("{lead} {}", self.version)
    }

    pub fn question(&self, strings: &Strings) -> String {
        let title = match self.ready {
            Some(_) => strings.restart_question,
            None => strings.update_question,
        };
        title.replace(VERSION_SLOT, &self.version)
    }
}

impl Running {
    pub fn offer_update(&mut self, offer: Offer) {
        self.update_offer = Some(offer);
        self.changed();
    }

    pub(in crate::app::running) fn restart_for_update(&mut self, event_loop: &ActiveEventLoop) {
        let Some(offer) = self.update_offer.clone() else {
            return;
        };
        let start = || match &offer.ready {
            Some(exe) => update::relaunch(exe),
            None => update::restart(&offer.version),
        };
        self.restart_with(event_loop, start, "the update restart did not start");
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

    fn offer(ready: bool) -> Offer {
        Offer {
            version: "9.8.7".to_owned(),
            ready: ready.then(|| PathBuf::from("trame.exe")),
        }
    }

    #[test]
    fn an_offer_still_to_download_asks_to_update() {
        let strings = Language::English.strings();
        assert_eq!(offer(false).label(strings), "Update to 9.8.7");
        assert_eq!(offer(false).question(strings), "Update to 9.8.7?");
    }

    #[test]
    fn a_version_already_in_place_only_asks_to_restart() {
        let strings = Language::English.strings();
        assert_eq!(offer(true).label(strings), "Restart into 9.8.7");
        assert_eq!(offer(true).question(strings), "Restart into 9.8.7?");
    }
}
