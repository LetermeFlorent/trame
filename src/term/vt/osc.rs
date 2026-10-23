use super::Perform;

const MAX_OSC_LEN: usize = 1 << 20;
const MAX_OSC_PARAMS: usize = 16;
const KEPT_CAPACITY: usize = 4096;

#[derive(Debug, Default)]
pub(super) struct OscBuffer {
    data: Vec<u8>,
    bounds: Vec<usize>,
    overflow: bool,
}

impl OscBuffer {
    pub(super) fn clear(&mut self) {
        self.data.clear();
        self.data.shrink_to(KEPT_CAPACITY);
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

    pub(super) fn dispatch<P: Perform>(&mut self, performer: &mut P, bell_terminated: bool) {
        if !self.overflow {
            let mut params = Vec::with_capacity(self.bounds.len() + 1);
            let mut start = 0;
            for &end in &self.bounds {
                params.push(&self.data[start..end]);
                start = end;
            }
            params.push(&self.data[start..]);
            performer.osc_dispatch(&params, bell_terminated);
        }
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::KEPT_CAPACITY;
    use crate::term::Screen;
    use crate::term::vt::Parser;

    #[test]
    fn a_long_osc_gives_its_memory_back_once_handled() {
        let mut parser = Parser::new();
        let mut screen = Screen::new(4, 20, 0);
        let title = "x".repeat(KEPT_CAPACITY * 8);
        parser.advance(&mut screen, format!("\x1b]2;{title}\x07").as_bytes());
        assert!(screen.title().is_some_and(|shown| shown.starts_with("xxx")));
        assert!(parser.osc.data.capacity() <= KEPT_CAPACITY);
    }
}
