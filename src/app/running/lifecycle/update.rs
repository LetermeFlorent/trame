use std::io;
use std::path::PathBuf;

use winit::event_loop::ActiveEventLoop;

use trame::locale::Strings;
use trame::log_error;

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

#[derive(Default)]
pub(in crate::app::running) struct UpdateState {
    pub(in crate::app::running) offer: Option<Offer>,
    pub(in crate::app::running) failed: bool,
}

impl Running {
    pub fn offer_update(&mut self, offer: Offer) {
        self.update.offer = Some(offer);
        self.changed();
    }

    pub(in crate::app::running) fn restart_for_update(&mut self, event_loop: &ActiveEventLoop) {
        let Some(offer) = self.update.offer.clone() else {
            return;
        };
        let start = || match &offer.ready {
            Some(exe) => update::relaunch(exe),
            None => update::restart(&offer.version),
        };
        if let Err(error) = self.restart_with(event_loop, start) {
            log_error!("the update restart did not start: {error}");
            self.update.failed = true;
            self.changed();
        }
    }

    pub(in crate::app::running) fn restart_if_card_lost(&mut self, event_loop: &ActiveEventLoop) {
        if self.restarting || !self.graphics.is_lost() {
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
        self.restarting = true;
        event_loop.exit();
        Ok(())
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
