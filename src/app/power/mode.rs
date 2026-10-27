#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Normal,
    Own,
    Family,
}

impl Mode {
    pub(super) fn wanted(enabled: bool, background: bool, idle: bool) -> Self {
        match (enabled, background, idle) {
            (false, _, _) | (true, false, false) => Mode::Normal,
            (true, true, _) => Mode::Family,
            (true, false, true) => Mode::Own,
        }
    }
}
