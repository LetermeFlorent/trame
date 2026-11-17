use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Transmit,
    TransmitPlace,
    Place,
    Delete,
    Query,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Rgb,
    Rgba,
    Png,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub action: Action,
    pub format: Format,
    pub transport: u8,
    pub compressed: bool,
    pub more: bool,
    pub id: u32,
    pub number: u32,
    pub placement: u32,
    pub quiet: u8,
    pub width: u32,
    pub height: u32,
    pub cols: u16,
    pub rows: u16,
    pub crop: Crop,
    pub offset: (u16, u16),
    pub cursor_stays: bool,
    pub z: i32,
    pub virtual_: bool,
    pub delete: u8,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            action: Action::Transmit,
            format: Format::Rgba,
            transport: b'd',
            compressed: false,
            more: false,
            id: 0,
            number: 0,
            placement: 0,
            quiet: 0,
            width: 0,
            height: 0,
            cols: 0,
            rows: 0,
            crop: Crop::default(),
            offset: (0, 0),
            cursor_stays: false,
            z: 0,
            virtual_: false,
            delete: b'a',
        }
    }
}

pub fn parse(control: &[u8]) -> Command {
    let mut command = Command::default();
    for pair in control.split(|&byte| byte == b',') {
        if let [key, b'=', value @ ..] = pair {
            apply(&mut command, *key, value);
        }
    }
    command
}

fn apply(command: &mut Command, key: u8, value: &[u8]) {
    let letter = value.first().copied().unwrap_or_default();
    match key {
        b'a' => command.action = action(letter),
        b'f' => command.format = format(number(value)),
        b't' => command.transport = letter,
        b'o' => command.compressed = letter == b'z',
        b'm' => command.more = number::<u8>(value) == 1,
        b'i' => command.id = number(value),
        b'I' => command.number = number(value),
        b'p' => command.placement = number(value),
        b'q' => command.quiet = number(value),
        b's' => command.width = number(value),
        b'v' => command.height = number(value),
        b'c' => command.cols = number(value),
        b'r' => command.rows = number(value),
        b'x' => command.crop.x = number(value),
        b'y' => command.crop.y = number(value),
        b'w' => command.crop.w = number(value),
        b'h' => command.crop.h = number(value),
        b'X' => command.offset.0 = number(value),
        b'Y' => command.offset.1 = number(value),
        b'C' => command.cursor_stays = number::<u8>(value) == 1,
        b'z' => command.z = number(value),
        b'U' => command.virtual_ = number::<u8>(value) == 1,
        b'd' => command.delete = letter,
        _ => {}
    }
}

fn number<T: FromStr + Default>(value: &[u8]) -> T {
    std::str::from_utf8(value)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or_default()
}

fn action(letter: u8) -> Action {
    match letter {
        b't' => Action::Transmit,
        b'T' => Action::TransmitPlace,
        b'p' => Action::Place,
        b'd' => Action::Delete,
        b'q' => Action::Query,
        _ => Action::Unsupported,
    }
}

fn format(value: u32) -> Format {
    match value {
        24 => Format::Rgb,
        32 => Format::Rgba,
        100 => Format::Png,
        _ => Format::Unsupported,
    }
}
