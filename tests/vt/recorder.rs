use trame::vt::{Params, Perform};

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Print(char),
    Execute(u8),
    Csi(Vec<Vec<u16>>, Vec<u8>, u8),
    Esc(Vec<u8>, u8),
    Osc(Vec<Vec<u8>>, bool),
}

#[derive(Default)]
pub struct Recorder {
    pub events: Vec<Event>,
}

impl Perform for Recorder {
    fn print(&mut self, c: char) {
        self.events.push(Event::Print(c));
    }

    fn execute(&mut self, byte: u8) {
        self.events.push(Event::Execute(byte));
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], action: u8) {
        let groups = params.iter().map(<[u16]>::to_vec).collect();
        self.events
            .push(Event::Csi(groups, intermediates.to_vec(), action));
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], byte: u8) {
        self.events.push(Event::Esc(intermediates.to_vec(), byte));
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let params = params.iter().map(|param| param.to_vec()).collect();
        self.events.push(Event::Osc(params, bell_terminated));
    }
}
