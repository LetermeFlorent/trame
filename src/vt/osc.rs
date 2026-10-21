use super::Perform;

const MAX_OSC_LEN: usize = 1 << 20;
const MAX_OSC_PARAMS: usize = 16;

#[derive(Debug, Default)]
pub(super) struct OscBuffer {
    data: Vec<u8>,
    bounds: Vec<usize>,
    overflow: bool,
}

impl OscBuffer {
    pub(super) fn clear(&mut self) {
        self.data.clear();
        self.bounds.clear();
        self.overflow = false;
    }

    pub(super) fn push(&mut self, byte: u8) {
        if self.data.len() >= MAX_OSC_LEN {
            self.overflow = true;
            return;
        }
        if byte == b';' && self.bounds.len() < MAX_OSC_PARAMS - 1 {
            self.bounds.push(self.data.len());
            return;
        }
        self.data.push(byte);
    }

    pub(super) fn dispatch<P: Perform>(&self, performer: &mut P, bell_terminated: bool) {
        if self.overflow {
            return;
        }
        let mut params = Vec::with_capacity(self.bounds.len() + 1);
        let mut start = 0;
        for &end in &self.bounds {
            params.push(&self.data[start..end]);
            start = end;
        }
        params.push(&self.data[start..]);
        performer.osc_dispatch(&params, bell_terminated);
    }
}
