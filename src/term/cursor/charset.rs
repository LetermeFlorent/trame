#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Charset {
    #[default]
    Ascii,
    LineDrawing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CharsetState {
    pub slots: [Charset; 2],
    pub active: usize,
}

impl CharsetState {
    pub fn map(&self, c: char) -> char {
        match self.slots[self.active] {
            Charset::Ascii => c,
            Charset::LineDrawing => line_drawing(c),
        }
    }

    pub fn designate(&mut self, slot: usize, designator: u8) {
        self.slots[slot] = match designator {
            b'0' => Charset::LineDrawing,
            _ => Charset::Ascii,
        };
    }
}

fn line_drawing(c: char) -> char {
    match c {
        '`' => '◆',
        'a' => '▒',
        'b' => '␉',
        'c' => '␌',
        'd' => '␍',
        'e' => '␊',
        'f' => '°',
        'g' => '±',
        'h' => '␤',
        'i' => '␋',
        'j' => '┘',
        'k' => '┐',
        'l' => '┌',
        'm' => '└',
        'n' => '┼',
        'o' => '⎺',
        'p' => '⎻',
        'q' => '─',
        'r' => '⎼',
        's' => '⎽',
        't' => '├',
        'u' => '┤',
        'v' => '┴',
        'w' => '┬',
        'x' => '│',
        'y' => '≤',
        'z' => '≥',
        '{' => 'π',
        '|' => '≠',
        '}' => '£',
        '~' => '·',
        _ => c,
    }
}
