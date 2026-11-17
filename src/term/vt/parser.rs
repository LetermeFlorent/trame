use super::osc::OscBuffer;
use super::utf8::{Decoded, Utf8Decoder};
use super::{Params, Perform};

pub(super) const MAX_INTERMEDIATES: usize = 2;
pub(super) const ESC: u8 = 0x1B;
pub(super) const DEL: u8 = 0x7F;
const CAN: u8 = 0x18;
const SUB: u8 = 0x1A;
pub(super) const C0_FIRST: u8 = 0x00;
pub(super) const C0_LAST: u8 = 0x1F;
pub(super) const INTERMEDIATE_FIRST: u8 = 0x20;
pub(super) const INTERMEDIATE_LAST: u8 = 0x2F;
const PRINTABLE_FIRST: u8 = 0x20;
const PRINTABLE_LAST: u8 = 0x7E;
const NON_ASCII_FIRST: u8 = 0x80;
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

impl State {
    fn breaks_on_text(self) -> bool {
        !matches!(self, Self::Ground | Self::OscString | Self::IgnoreString)
    }
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
        let mut rest = bytes;
        while let Some((&byte, tail)) = rest.split_first() {
            let taken = if self.state == State::Ground {
                let run = self.ascii_run(rest);
                if run > 0 {
                    performer.print_ascii(&rest[..run]);
                }
                run
            } else {
                self.csi_run(performer, rest)
            };
            if taken == 0 {
                self.advance_byte(performer, byte);
                rest = tail;
            } else {
                rest = &rest[taken..];
            }
        }
    }

    fn advance_byte<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        if byte >= NON_ASCII_FIRST && self.state.breaks_on_text() {
            self.state = State::Ground;
        }
        if self.state == State::Ground && (byte >= NON_ASCII_FIRST || self.utf8.is_pending()) {
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
            C0_FIRST..=C0_LAST => performer.execute(byte),
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

    fn ascii_run(&self, bytes: &[u8]) -> usize {
        if self.state != State::Ground || self.utf8.is_pending() {
            return 0;
        }
        bytes
            .iter()
            .position(|byte| !(PRINTABLE_FIRST..=PRINTABLE_LAST).contains(byte))
            .unwrap_or(bytes.len())
    }
}
