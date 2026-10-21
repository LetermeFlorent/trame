use super::osc::OscBuffer;
use super::utf8::{Decoded, Utf8Decoder};
use super::{Params, Perform};

pub(super) const MAX_INTERMEDIATES: usize = 2;
pub(super) const ESC: u8 = 0x1B;
pub(super) const DEL: u8 = 0x7F;
const CAN: u8 = 0x18;
const SUB: u8 = 0x1A;
const BEL: u8 = 0x07;
const REPLACEMENT: char = '\u{FFFD}';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum State {
    #[default]
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,
    OscString,
    IgnoreString,
}

#[derive(Debug, Default)]
pub struct Parser {
    pub(super) state: State,
    pub(super) params: Params,
    pub(super) param: u16,
    pub(super) has_params: bool,
    pub(super) next_is_subparam: bool,
    pub(super) intermediates: [u8; MAX_INTERMEDIATES],
    pub(super) intermediate_len: usize,
    pub(super) ignoring: bool,
    pub(super) osc: OscBuffer,
    utf8: Utf8Decoder,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance<P: Perform>(&mut self, performer: &mut P, bytes: &[u8]) {
        for &byte in bytes {
            self.advance_byte(performer, byte);
        }
    }

    fn advance_byte<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        if self.state == State::Ground && (byte >= 0x80 || self.utf8.is_pending()) {
            self.decode_utf8(performer, byte);
            return;
        }
        match byte {
            CAN | SUB => {
                performer.execute(byte);
                self.state = State::Ground;
            }
            ESC => self.enter_escape(performer),
            _ => self.dispatch_state(performer, byte),
        }
    }

    fn dispatch_state<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match self.state {
            State::Ground => self.ground(performer, byte),
            State::Escape => self.escape(performer, byte),
            State::EscapeIntermediate => self.escape_intermediate(performer, byte),
            State::CsiEntry | State::CsiParam | State::CsiIntermediate | State::CsiIgnore => {
                self.csi(performer, byte)
            }
            State::OscString => self.osc_byte(performer, byte),
            State::IgnoreString => {}
        }
    }

    fn ground<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            0x00..=0x1F => performer.execute(byte),
            DEL => {}
            _ => performer.print(char::from(byte)),
        }
    }

    fn decode_utf8<P: Perform>(&mut self, performer: &mut P, byte: u8) {
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

    fn enter_escape<P: Perform>(&mut self, performer: &mut P) {
        if self.state == State::OscString {
            self.osc.dispatch(performer, false);
        }
        self.intermediate_len = 0;
        self.ignoring = false;
        self.state = State::Escape;
    }

    fn osc_byte<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            BEL => {
                self.osc.dispatch(performer, true);
                self.state = State::Ground;
            }
            0x00..=0x1F => {}
            _ => self.osc.push(byte),
        }
    }

    pub(super) fn collect(&mut self, byte: u8) {
        if self.intermediate_len == MAX_INTERMEDIATES {
            self.ignoring = true;
            return;
        }
        self.intermediates[self.intermediate_len] = byte;
        self.intermediate_len += 1;
    }

    pub(super) fn intermediates(&self) -> &[u8] {
        &self.intermediates[..self.intermediate_len]
    }
}
