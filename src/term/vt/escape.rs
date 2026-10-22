use super::Perform;
use super::parser::{DEL, Parser, State};

impl Parser {
    pub(super) fn escape<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            0x00..=0x1F => performer.execute(byte),
            0x20..=0x2F => {
                self.collect(byte);
                self.state = State::EscapeIntermediate;
            }
            b'[' => self.enter_csi(),
            b']' => {
                self.osc.clear();
                self.state = State::OscString;
            }
            b'P' | b'X' | b'^' | b'_' => self.state = State::IgnoreString,
            DEL => {}
            _ => {
                performer.esc_dispatch(self.intermediates(), byte);
                self.state = State::Ground;
            }
        }
    }

    pub(super) fn escape_intermediate<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            0x00..=0x1F => performer.execute(byte),
            0x20..=0x2F => self.collect(byte),
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
