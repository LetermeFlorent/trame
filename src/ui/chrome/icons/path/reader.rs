pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            at: 0,
        }
    }

    fn skip(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b',')
        {
            self.at += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    pub(super) fn done(&mut self) -> bool {
        self.skip();
        self.peek().is_none()
    }

    pub(super) fn command(&mut self) -> Option<u8> {
        self.skip();
        let byte = self.peek().filter(u8::is_ascii_alphabetic)?;
        self.at += 1;
        Some(byte)
    }

    pub(super) fn flag(&mut self) -> Option<bool> {
        self.skip();
        let flag = match self.peek()? {
            b'0' => false,
            b'1' => true,
            _ => return None,
        };
        self.at += 1;
        Some(flag)
    }

    fn digits(&mut self) -> usize {
        let from = self.at;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.at += 1;
        }
        self.at - from
    }

    pub(super) fn number(&mut self) -> Option<f32> {
        self.skip();
        let from = self.at;
        if matches!(self.peek(), Some(b'-' | b'+')) {
            self.at += 1;
        }
        let mut count = self.digits();
        if self.peek() == Some(b'.') {
            self.at += 1;
            count += self.digits();
        }
        if count == 0 {
            self.at = from;
            return None;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            let mark = self.at;
            self.at += 1;
            if matches!(self.peek(), Some(b'-' | b'+')) {
                self.at += 1;
            }
            if self.digits() == 0 {
                self.at = mark;
            }
        }
        std::str::from_utf8(&self.bytes[from..self.at])
            .ok()?
            .parse()
            .ok()
    }
}
