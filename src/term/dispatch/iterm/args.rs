#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Size {
    Auto,
    Cells(u32),
    Pixels(u32),
    Percent(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FileArgs {
    pub(crate) inline: bool,
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) preserve: bool,
    pub(crate) stay: bool,
}

impl Default for FileArgs {
    fn default() -> Self {
        Self {
            inline: false,
            width: Size::Auto,
            height: Size::Auto,
            preserve: true,
            stay: false,
        }
    }
}

pub(crate) fn parse(args: &[u8]) -> FileArgs {
    let mut parsed = FileArgs::default();
    for pair in args.split(|&byte| byte == b';') {
        let Some(equals) = pair.iter().position(|&byte| byte == b'=') else {
            continue;
        };
        let (key, value) = (&pair[..equals], &pair[equals + 1..]);
        match key {
            b"inline" => parsed.inline = value == b"1",
            b"width" => parsed.width = size(value),
            b"height" => parsed.height = size(value),
            b"preserveAspectRatio" => parsed.preserve = value != b"0",
            b"doNotMoveCursor" => parsed.stay = value == b"1",
            _ => {}
        }
    }
    parsed
}

fn size(value: &[u8]) -> Size {
    let text = String::from_utf8_lossy(value);
    let number = |digits: &str| digits.parse::<u32>().ok();
    let parsed = if let Some(digits) = text.strip_suffix("px") {
        number(digits).map(Size::Pixels)
    } else if let Some(digits) = text.strip_suffix('%') {
        number(digits).map(Size::Percent)
    } else {
        number(&text).map(Size::Cells)
    };
    match parsed {
        Some(Size::Cells(0) | Size::Pixels(0) | Size::Percent(0)) | None => Size::Auto,
        Some(size) => size,
    }
}
