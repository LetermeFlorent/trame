use winit::window::WindowId;

use trame::preferences::Preferences;

use crate::app::running::Running;

impl Running {
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn slot(&self) -> usize {
        self.saver.slot().index()
    }

    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    pub fn follow_preferences(&mut self, preferences: &Preferences) {
        if preferences != &self.preferences {
            self.set_preferences(preferences.clone());
        }
    }

    pub fn rests(&self) -> bool {
        self.power.rests()
    }

    pub fn hidden(&self) -> bool {
        self.power.hidden()
    }

    pub fn bring_forward(&self) {
        self.window.focus_window();
    }

    pub fn retire(&mut self) {
        self.transcripts.flush();
    }
}
