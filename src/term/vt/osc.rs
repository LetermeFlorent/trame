use super::Perform;
use super::parser::{C0_FIRST, C0_LAST, Parser, State};

const BEL: u8 = 0x07;
const MAX_OSC_LEN: usize = 1 << 20;
const MAX_IMAGE_OSC_LEN: usize = 64 << 20;
const IMAGE_KIND: &[u8] = b"1337";
const IMAGE_PREFIXES: [&[u8]; 2] = [b"File=", b"FilePart="];
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
        if byte == b';' && self.bounds.len() < MAX_OSC_PARAMS - 1 {
            self.bounds.push(self.data.len());
            return;
        }
        self.extend(&[byte]);
    }

    fn extend(&mut self, bytes: &[u8]) {
        let len = self.data.len() + bytes.len();
        if self.overflow
            || (len > MAX_OSC_LEN && (len > MAX_IMAGE_OSC_LEN || !self.is_image(bytes)))
        {
            self.overflow = true;
            return;
        }
        self.data.extend_from_slice(bytes);
    }

    fn is_image(&self, incoming: &[u8]) -> bool {
        let Some(&kind_end) = self.bounds.first() else {
            return false;
        };
        let rest = || self.data[kind_end..].iter().chain(incoming);
        &self.data[..kind_end] == IMAGE_KIND
            && IMAGE_PREFIXES
                .iter()
                .any(|prefix| prefix.iter().eq(rest().take(prefix.len())))
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

impl Parser {
    pub(super) fn osc_byte<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            BEL => {
                self.osc.dispatch(performer, true);
                self.state = State::Ground;
            }
            C0_FIRST..=C0_LAST => {}
            _ => self.osc.push(byte),
        }
    }

    #[inline(never)]
    pub(super) fn osc_run(&mut self, bytes: &[u8]) -> usize {
        let run = bytes
            .iter()
            .position(|&byte| byte == b';' || (C0_FIRST..=C0_LAST).contains(&byte))
            .unwrap_or(bytes.len());
        self.osc.extend(&bytes[..run]);
        run
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
