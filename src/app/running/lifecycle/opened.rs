use std::error::Error;
use std::path::PathBuf;

use winit::dpi::PhysicalPosition;

use trame::desk::{Placement, TornTab};
use trame::host::update::UpdateStatus;
use trame::preferences::Recall;
use trame::workspace::Snapshot;

use super::super::Running;
use crate::app::window;

const TORN_OFFSET: f32 = 48.0;

pub struct Opened {
    pub(super) running: Running,
    pub(super) saved: Option<Snapshot>,
}

impl Opened {
    pub fn show(
        self,
        update: &UpdateStatus,
        folder: Option<PathBuf>,
    ) -> Result<Running, Box<dyn Error>> {
        let Opened { mut running, saved } = self;
        running.set_update(update);
        if running.loader.is_some() {
            running.draw();
            running.window.set_visible(true);
        }
        let restored = running.desk.saver.owns() && running.restore_session(saved.as_ref());
        if !restored || folder.is_some() {
            let effects = running.desk.open_at(Placement::Instance, folder);
            running.apply(effects);
            if running.desk.workspace.is_empty() {
                return Err("the first shell could not start".into());
            }
        }
        if !running.desk.preferences.recalls(Recall::Text) {
            running.desk.transcripts.clear();
        }
        running.draw();
        running.window.set_visible(true);
        Ok(running)
    }

    pub fn adopt(self, update: &UpdateStatus, torn: TornTab, at: PhysicalPosition<i32>) -> Running {
        let mut running = self.running;
        running.set_update(update);
        let offset = running.metrics.px(TORN_OFFSET).round() as i32;
        let corner = window::place_near(&running.window, at, offset);
        running.window.set_outer_position(corner);
        running.take_torn_tab(torn);
        running.draw();
        running.window.set_visible(true);
        running
    }
}
