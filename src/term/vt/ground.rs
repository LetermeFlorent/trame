use super::Perform;
use super::parser::{C0_FIRST, C0_LAST, DEL, Parser, State};
use super::utf8::Decoded;

const PRINTABLE_FIRST: u8 = 0x20;
const PRINTABLE_LAST: u8 = 0x7E;
const REPLACEMENT: char = '\u{FFFD}';

impl Parser {
    pub(super) fn ground<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            C0_FIRST..=C0_LAST => performer.execute(byte),
            DEL => {}
            _ => performer.print(char::from(byte)),
        }
    }

    pub(super) fn decode_utf8<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match self.utf8.feed(byte) {
            Decoded::Char(c) => performer.print(c),
            Decoded::Pending => {}
            Decoded::Replaced => performer.print(REPLACEMENT),
            Decoded::ReplacedAndRetry => {
                performer.print(REPLACEMENT);
                self.advance_byte(performer, byte);
            }
        }
    }

    pub(super) fn ascii_run(&self, bytes: &[u8]) -> usize {
        if self.state != State::Ground || self.utf8.is_pending() {
            return 0;
        }
        bytes
            .iter()
            .position(|byte| !(PRINTABLE_FIRST..=PRINTABLE_LAST).contains(byte))
            .unwrap_or(bytes.len())
    }
}
