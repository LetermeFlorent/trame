use super::codes::sgr;
use crate::term::cell::{Cell, Color, Flags};
use crate::term::screen::Screen;
use crate::term::vt::{Params, ParamsIter};

impl Screen {
    pub(crate) fn apply_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            self.cursor.template = Cell::default();
            return;
        }
        let mut groups = params.iter();
        while let Some(group) = groups.next() {
            apply_group(&mut self.cursor.template, group, &mut groups);
        }
    }
}

fn apply_group(template: &mut Cell, group: &[u16], rest: &mut ParamsIter) {
    let flags = &mut template.flags;
    match group[0] {
        sgr::RESET => *template = Cell::default(),
        sgr::BOLD => flags.insert(Flags::BOLD),
        sgr::DIM => flags.insert(Flags::DIM),
        sgr::ITALIC => flags.insert(Flags::ITALIC),
        sgr::UNDERLINE => set_underline(
            flags,
            group.get(1).copied().unwrap_or(sgr::UNDERLINE_SINGLE),
        ),
        sgr::INVERSE => flags.insert(Flags::INVERSE),
        sgr::HIDDEN => flags.insert(Flags::HIDDEN),
        sgr::STRIKE => flags.insert(Flags::STRIKE),
        sgr::DOUBLE_UNDERLINE => set_underline(flags, sgr::UNDERLINE_DOUBLE),
        sgr::NORMAL_INTENSITY => {
            flags.remove(Flags::BOLD);
            flags.remove(Flags::DIM);
        }
        sgr::NO_ITALIC => flags.remove(Flags::ITALIC),
        sgr::NO_UNDERLINE => flags.remove(Flags::ANY_UNDERLINE),
        sgr::NO_INVERSE => flags.remove(Flags::INVERSE),
        sgr::NO_HIDDEN => flags.remove(Flags::HIDDEN),
        sgr::NO_STRIKE => flags.remove(Flags::STRIKE),
        _ => apply_color_code(template, group, rest),
    }
}

fn apply_color_code(template: &mut Cell, group: &[u16], rest: &mut ParamsIter) {
    match group[0] {
        value @ sgr::FOREGROUND_FIRST..=sgr::FOREGROUND_LAST => {
            template.fg = Color::Indexed((value - sgr::FOREGROUND_FIRST) as u8)
        }
        sgr::FOREGROUND_EXTENDED => {
            template.fg = extended_color(group, rest).unwrap_or(template.fg)
        }
        sgr::FOREGROUND_DEFAULT => template.fg = Color::Foreground,
        value @ sgr::BACKGROUND_FIRST..=sgr::BACKGROUND_LAST => {
            template.bg = Color::Indexed((value - sgr::BACKGROUND_FIRST) as u8)
        }
        sgr::BACKGROUND_EXTENDED => {
            template.bg = extended_color(group, rest).unwrap_or(template.bg)
        }
        sgr::BACKGROUND_DEFAULT => template.bg = Color::Background,
        sgr::UNDERLINE_COLOR => {
            extended_color(group, rest);
        }
        value @ sgr::BRIGHT_FOREGROUND_FIRST..=sgr::BRIGHT_FOREGROUND_LAST => {
            template.fg =
                Color::Indexed((value - sgr::BRIGHT_FOREGROUND_FIRST + sgr::BRIGHT_OFFSET) as u8)
        }
        value @ sgr::BRIGHT_BACKGROUND_FIRST..=sgr::BRIGHT_BACKGROUND_LAST => {
            template.bg =
                Color::Indexed((value - sgr::BRIGHT_BACKGROUND_FIRST + sgr::BRIGHT_OFFSET) as u8)
        }
        _ => {}
    }
}

fn set_underline(flags: &mut Flags, style: u16) {
    flags.remove(Flags::ANY_UNDERLINE);
    match style {
        sgr::UNDERLINE_NONE => {}
        sgr::UNDERLINE_DOUBLE => flags.insert(Flags::DOUBLE_UNDERLINE),
        sgr::UNDERLINE_CURLY => flags.insert(Flags::CURLY_UNDERLINE),
        _ => flags.insert(Flags::UNDERLINE),
    }
}

fn extended_color(group: &[u16], rest: &mut ParamsIter) -> Option<Color> {
    if group.len() > 1 {
        return colon_color(&group[1..]);
    }
    let kind = *rest.next()?.first()?;
    match kind {
        sgr::COLOR_INDEXED => Some(Color::Indexed(channel(*rest.next()?.first()?))),
        sgr::COLOR_RGB => {
            let r = *rest.next()?.first()?;
            let g = *rest.next()?.first()?;
            let b = *rest.next()?.first()?;
            Some(Color::Rgb(channel(r), channel(g), channel(b)))
        }
        _ => None,
    }
}

fn colon_color(values: &[u16]) -> Option<Color> {
    match *values.first()? {
        sgr::COLOR_INDEXED => values.get(1).map(|&index| Color::Indexed(channel(index))),
        sgr::COLOR_RGB => {
            let rgb = match values.len() {
                sgr::COLON_RGB_WITH_SPACE_LEN.. => &values[values.len() - sgr::RGB_PARTS..],
                sgr::COLON_RGB_LEN => &values[1..=sgr::RGB_PARTS],
                _ => return None,
            };
            Some(Color::Rgb(
                channel(rgb[0]),
                channel(rgb[1]),
                channel(rgb[2]),
            ))
        }
        _ => None,
    }
}

fn channel(value: u16) -> u8 {
    value.min(u16::from(u8::MAX)) as u8
}
