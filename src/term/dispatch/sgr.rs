use crate::term::cell::{Cell, Color, Flags};
use crate::term::screen::Screen;
use crate::vt::{Params, ParamsIter};

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
        0 => *template = Cell::default(),
        1 => flags.insert(Flags::BOLD),
        2 => flags.insert(Flags::DIM),
        3 => flags.insert(Flags::ITALIC),
        4 => set_underline(flags, group.get(1).copied().unwrap_or(1)),
        7 => flags.insert(Flags::INVERSE),
        8 => flags.insert(Flags::HIDDEN),
        9 => flags.insert(Flags::STRIKE),
        21 => set_underline(flags, 2),
        22 => {
            flags.remove(Flags::BOLD);
            flags.remove(Flags::DIM);
        }
        23 => flags.remove(Flags::ITALIC),
        24 => flags.remove(Flags::ANY_UNDERLINE),
        27 => flags.remove(Flags::INVERSE),
        28 => flags.remove(Flags::HIDDEN),
        29 => flags.remove(Flags::STRIKE),
        value @ 30..=37 => template.fg = Color::Indexed((value - 30) as u8),
        38 => {
            if let Some(color) = extended_color(group, rest) {
                template.fg = color;
            }
        }
        39 => template.fg = Color::Foreground,
        value @ 40..=47 => template.bg = Color::Indexed((value - 40) as u8),
        48 => {
            if let Some(color) = extended_color(group, rest) {
                template.bg = color;
            }
        }
        49 => template.bg = Color::Background,
        58 => {
            extended_color(group, rest);
        }
        value @ 90..=97 => template.fg = Color::Indexed((value - 90 + 8) as u8),
        value @ 100..=107 => template.bg = Color::Indexed((value - 100 + 8) as u8),
        _ => {}
    }
}

fn set_underline(flags: &mut Flags, style: u16) {
    flags.remove(Flags::ANY_UNDERLINE);
    match style {
        0 => {}
        2 => flags.insert(Flags::DOUBLE_UNDERLINE),
        3 => flags.insert(Flags::CURLY_UNDERLINE),
        _ => flags.insert(Flags::UNDERLINE),
    }
}

fn extended_color(group: &[u16], rest: &mut ParamsIter) -> Option<Color> {
    if group.len() > 1 {
        return colon_color(&group[1..]);
    }
    let kind = *rest.next()?.first()?;
    match kind {
        5 => Some(Color::Indexed(channel(*rest.next()?.first()?))),
        2 => {
            let r = *rest.next()?.first()?;
            let g = *rest.next()?.first()?;
            let b = *rest.next()?.first()?;
            Some(Color::Rgb(channel(r), channel(g), channel(b)))
        }
        _ => None,
    }
}

fn colon_color(values: &[u16]) -> Option<Color> {
    match values.first()? {
        5 => values.get(1).map(|&index| Color::Indexed(channel(index))),
        2 => {
            let rgb = match values.len() {
                5.. => &values[values.len() - 3..],
                4 => &values[1..4],
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
    value.min(255) as u8
}
