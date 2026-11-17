use super::Perform;
use super::parser::{C0_FIRST, C0_LAST, Parser};

const MAX_APC_LEN: usize = 1 << 20;
const KEPT_CAPACITY: usize = 4096;

#[derive(Debug, Default)]
pub(super) struct ApcBuffer {
    data: Vec<u8>,
    overflow: bool,
}

impl ApcBuffer {
    pub(super) fn clear(&mut self) {
        self.data.clear();
        self.data.shrink_to(KEPT_CAPACITY);
        self.overflow = false;
    }

    pub(super) fn push(&mut self, byte: u8) {
        self.extend(&[byte]);
    }

    pub(super) fn extend(&mut self, bytes: &[u8]) {
        if self.overflow || self.data.len() + bytes.len() > MAX_APC_LEN {
            self.overflow = true;
            return;
        }
        self.data.extend_from_slice(bytes);
    }

    pub(super) fn take(&self) -> Option<&[u8]> {
        (!self.overflow).then_some(self.data.as_slice())
    }
}

impl Parser {
    pub(super) fn apc_byte(&mut self, byte: u8) {
        if !(C0_FIRST..=C0_LAST).contains(&byte) {
            self.apc.push(byte);
        }
    }

    #[inline(never)]
    pub(super) fn apc_run(&mut self, bytes: &[u8]) -> usize {
        let run = bytes
            .iter()
            .position(|byte| (C0_FIRST..=C0_LAST).contains(byte))
            .unwrap_or(bytes.len());
        self.apc.extend(&bytes[..run]);
        run
    }

    pub(super) fn apc_dispatch<P: Perform>(&mut self, performer: &mut P) {
        if let Some(data) = self.apc.take() {
            performer.apc_dispatch(data);
        }
        self.apc.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{KEPT_CAPACITY, MAX_APC_LEN};
    use crate::term::Screen;
    use crate::term::vt::Parser;

    #[test]
    fn a_long_apc_gives_its_memory_back_once_handled() {
        let mut parser = Parser::new();
        let mut screen = Screen::new(4, 20, 0);
        let payload = "A".repeat(KEPT_CAPACITY * 8);
        parser.advance(&mut screen, format!("\x1b_G{payload}\x1b\\").as_bytes());
        assert!(parser.apc.data.capacity() <= KEPT_CAPACITY);
    }

    #[test]
    fn take_refuses_an_overflowed_buffer() {
        let mut parser = Parser::new();
        parser.apc.extend(&vec![b'A'; MAX_APC_LEN]);
        assert!(parser.apc.take().is_some());
        parser.apc.push(b'A');
        assert!(parser.apc.take().is_none());
    }
}
