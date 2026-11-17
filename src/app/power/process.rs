use super::set_own;

#[derive(Default)]
pub struct Process {
    state: (bool, bool),
}

impl Process {
    pub fn follow(&mut self, resting: bool, hidden: bool) {
        let state = (resting, resting && hidden);
        if self.state != state {
            self.state = state;
            set_own(state.0, state.1);
        }
    }
}
