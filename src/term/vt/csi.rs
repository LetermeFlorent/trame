use super::Perform;
use super::parser::{DEL, Parser, State};

impl Parser {
    pub(super) fn enter_csi(&mut self) {
        self.params.clear();
        self.param = 0;
        self.has_params = false;
        self.next_is_subparam = false;
        self.intermediate_len = 0;
        self.ignoring = false;
        self.state = State::CsiEntry;
    }

    pub(super) fn csi<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        match byte {
            0x00..=0x1F => performer.execute(byte),
            DEL => {}
            0x40..=0x7E => self.csi_final(performer, byte),
            _ if self.state == State::CsiIgnore => {}
            0x20..=0x2F => {
                self.collect(byte);
                self.state = State::CsiIntermediate;
            }
            b'0'..=b'9' | b';' | b':' => self.csi_param(byte),
            0x3C..=0x3F => self.csi_marker(byte),
            _ => {}
        }
    }

    fn csi_param(&mut self, byte: u8) {
        if self.state == State::CsiIntermediate {
            self.state = State::CsiIgnore;
            return;
        }
        self.state = State::CsiParam;
        self.has_params = true;
        match byte {
            b';' => {
                self.finish_param();
                self.next_is_subparam = false;
            }
            b':' => {
                self.finish_param();
                self.next_is_subparam = true;
            }
            _ => {
                let digit = u16::from(byte - b'0');
                self.param = self.param.saturating_mul(10).saturating_add(digit);
            }
        }
    }

    fn csi_marker(&mut self, byte: u8) {
        if self.state == State::CsiEntry {
            self.collect(byte);
            self.state = State::CsiParam;
        } else {
            self.state = State::CsiIgnore;
        }
    }

    fn finish_param(&mut self) {
        if self.params.is_full() {
            self.ignoring = true;
            return;
        }
        if self.next_is_subparam && !self.params.is_empty() {
            self.params.extend(self.param);
        } else {
            self.params.push(self.param);
        }
        self.param = 0;
    }

    fn csi_final<P: Perform>(&mut self, performer: &mut P, byte: u8) {
        if self.state != State::CsiIgnore {
            if self.has_params {
                self.finish_param();
            }
            if !self.ignoring {
                performer.csi_dispatch(&self.params, self.intermediates(), byte);
            }
        }
        self.state = State::Ground;
    }
}
