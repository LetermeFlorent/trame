const CONTINUATION_MASK: u8 = 0xC0;
const CONTINUATION_TAG: u8 = 0x80;
const CONTINUATION_PAYLOAD: u8 = 0x3F;
const CONTINUATION_BITS: u32 = 6;
const TWO_BYTE_FIRST: u8 = 0xC2;
const TWO_BYTE_LAST: u8 = 0xDF;
const TWO_BYTE_PAYLOAD: u8 = 0x1F;
const TWO_BYTE_MINIMUM: u32 = 0x80;
const THREE_BYTE_FIRST: u8 = 0xE0;
const THREE_BYTE_LAST: u8 = 0xEF;
const THREE_BYTE_PAYLOAD: u8 = 0x0F;
const THREE_BYTE_MINIMUM: u32 = 0x800;
const FOUR_BYTE_FIRST: u8 = 0xF0;
const FOUR_BYTE_LAST: u8 = 0xF4;
const FOUR_BYTE_PAYLOAD: u8 = 0x07;
const FOUR_BYTE_MINIMUM: u32 = 0x1_0000;

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
        if byte & CONTINUATION_MASK != CONTINUATION_TAG {
            self.remaining = 0;
            return Decoded::ReplacedAndRetry;
        }
        self.codepoint =
            (self.codepoint << CONTINUATION_BITS) | u32::from(byte & CONTINUATION_PAYLOAD);
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
            TWO_BYTE_FIRST..=TWO_BYTE_LAST => (1, TWO_BYTE_PAYLOAD, TWO_BYTE_MINIMUM),
            THREE_BYTE_FIRST..=THREE_BYTE_LAST => (2, THREE_BYTE_PAYLOAD, THREE_BYTE_MINIMUM),
            FOUR_BYTE_FIRST..=FOUR_BYTE_LAST => (3, FOUR_BYTE_PAYLOAD, FOUR_BYTE_MINIMUM),
            _ => return Decoded::Replaced,
        };
        self.remaining = remaining;
        self.codepoint = u32::from(byte & mask);
        self.minimum = minimum;
        Decoded::Pending
    }
}
