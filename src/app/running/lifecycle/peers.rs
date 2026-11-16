use winit::window::WindowId;

use trame::preferences::Preferences;

use crate::app::running::Running;

impl Running {
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn slot(&self) -> usize {
        self.desk.saver.slot().index()
    }

    pub fn preferences(&self) -> &Preferences {
        &self.desk.preferences
    }

    pub fn follow_preferences(&mut self, preferences: &Preferences) {
        let effects = self.desk.follow_preferences(preferences);
        self.apply(effects);
    }

    pub fn rests(&self) -> bool {
        self.desk.power.rests()
    }

    pub fn hidden(&self) -> bool {
        self.desk.power.hidden()
    }

    pub fn bring_forward(&self) {
        self.window.focus_window();
    }

    pub fn retire(&mut self) {
        self.desk.transcripts.flush();
    }
}
