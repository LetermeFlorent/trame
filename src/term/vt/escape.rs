use super::Perform;
use super::parser::{
    C0_FIRST, C0_LAST, DEL, INTERMEDIATE_FIRST, INTERMEDIATE_LAST, MAX_INTERMEDIATES, Parser, State,
};

impl Parser {
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

    pub(super) fn escape<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            C0_FIRST..=C0_LAST => performer.execute(byte),
            INTERMEDIATE_FIRST..=INTERMEDIATE_LAST => {
                self.collect(byte);
                self.state = State::EscapeIntermediate;
            }
            b'[' => self.enter_csi(),
            b']' => {
                self.osc.clear();
                self.state = State::OscString;
            }
            b'_' => {
                self.apc.clear();
                self.state = State::ApcString;
            }
            b'P' | b'X' | b'^' => self.state = State::IgnoreString,
            DEL => {}
            _ => {
                performer.esc_dispatch(self.intermediates(), byte);
                self.state = State::Ground;
            }
        }
    }

    pub(super) fn escape_intermediate<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            C0_FIRST..=C0_LAST => performer.execute(byte),
            INTERMEDIATE_FIRST..=INTERMEDIATE_LAST => self.collect(byte),
            DEL => {}
            _ => {
                if !self.ignoring {
                    performer.esc_dispatch(self.intermediates(), byte);
                }
                self.state = State::Ground;
            }
        }
    }
}
