#[derive(Debug, PartialEq, Eq)]
pub(super) enum Decoded {
    Char(char),
    Pending,
    Replaced,
    ReplacedAndRetry,
}

#[derive(Debug, Default)]
pub(super) struct Utf8Decoder {
    codepoint: u32,
    remaining: u8,
    minimum: u32,
}

impl Utf8Decoder {
    pub(super) fn is_pending(&self) -> bool {
        self.remaining > 0
    }

    pub(super) fn feed(&mut self, byte: u8) -> Decoded {
        if self.remaining == 0 {
            return self.start(byte);
        }
        if byte & 0xC0 != 0x80 {
            self.remaining = 0;
            return Decoded::ReplacedAndRetry;
        }
        self.codepoint = (self.codepoint << 6) | u32::from(byte & 0x3F);
        self.remaining -= 1;
        if self.remaining > 0 {
            return Decoded::Pending;
        }
        if self.codepoint < self.minimum {
            return Decoded::Replaced;
        }
        char::from_u32(self.codepoint).map_or(Decoded::Replaced, Decoded::Char)
    }

    fn start(&mut self, byte: u8) -> Decoded {
        let (remaining, mask, minimum) = match byte {
            0xC2..=0xDF => (1, 0x1F, 0x80),
            0xE0..=0xEF => (2, 0x0F, 0x800),
            0xF0..=0xF4 => (3, 0x07, 0x1_0000),
            _ => return Decoded::Replaced,
        };
        self.remaining = remaining;
        self.codepoint = u32::from(byte & mask);
        self.minimum = minimum;
        Decoded::Pending
    }
}
