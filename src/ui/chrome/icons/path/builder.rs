use super::curves::{Arc, arc, cubic, quadratic};
use super::reader::Reader;
use crate::geometry::Point;

#[derive(Default)]
pub(super) struct Builder {
    pub(super) lines: Vec<Vec<Point>>,
    points: Vec<Point>,
    current: Point,
    start: Point,
    cubic_control: Option<Point>,
    quad_control: Option<Point>,
}

impl Builder {
    pub(super) fn finish(&mut self) {
        let points = std::mem::take(&mut self.points);
        if points.len() >= 2 {
            self.lines.push(points);
        }
    }

    fn extend(&mut self, points: Vec<Point>) {
        if self.points.is_empty() {
            self.points.push(self.current);
        }
        if let Some(last) = points.last() {
            self.current = *last;
        }
        self.points.extend(points);
    }

    fn reflect(&self, control: Option<Point>) -> Point {
        control.map_or(self.current, |c| {
            Point::new(2.0 * self.current.x - c.x, 2.0 * self.current.y - c.y)
        })
    }

    pub(super) fn step(&mut self, command: u8, reader: &mut Reader) -> Option<u8> {
        let base = match command.is_ascii_lowercase() {
            true => self.current,
            false => Point::default(),
        };
        let upper = command.to_ascii_uppercase();
        let next = match upper {
            b'M' | b'L' | b'H' | b'V' | b'Z' => self.line_command(command, base, reader)?,
            b'C' | b'S' | b'Q' | b'T' => {
                self.curve_command(upper, base, reader)?;
                command
            }
            b'A' => {
                self.arc_command(base, reader)?;
                command
            }
            _ => return None,
        };
        if !matches!(upper, b'C' | b'S') {
            self.cubic_control = None;
        }
        if !matches!(upper, b'Q' | b'T') {
            self.quad_control = None;
        }
        Some(next)
    }

    fn line_command(&mut self, command: u8, base: Point, reader: &mut Reader) -> Option<u8> {
        let mut next = command;
        match command.to_ascii_uppercase() {
            b'M' => {
                let point = at(base, reader)?;
                self.finish();
                self.current = point;
                self.start = point;
                next = if command == b'm' { b'l' } else { b'L' };
            }
            b'L' => self.extend(vec![at(base, reader)?]),
            b'H' => {
                let x = reader.number()? + base.x;
                self.extend(vec![Point::new(x, self.current.y)]);
            }
            b'V' => {
                let y = reader.number()? + base.y;
                self.extend(vec![Point::new(self.current.x, y)]);
            }
            _ => {
                if !self.points.is_empty() {
                    self.extend(vec![self.start]);
                }
                self.finish();
                self.current = self.start;
            }
        }
        Some(next)
    }

    fn curve_command(&mut self, upper: u8, base: Point, reader: &mut Reader) -> Option<()> {
        match upper {
            b'C' | b'S' => {
                let first = match upper {
                    b'C' => at(base, reader)?,
                    _ => self.reflect(self.cubic_control),
                };
                let second = at(base, reader)?;
                let to = at(base, reader)?;
                self.extend(cubic(self.current, first, second, to));
                self.cubic_control = Some(second);
            }
            _ => {
                let control = match upper {
                    b'Q' => at(base, reader)?,
                    _ => self.reflect(self.quad_control),
                };
                let to = at(base, reader)?;
                self.extend(quadratic(self.current, control, to));
                self.quad_control = Some(control);
            }
        }
        Some(())
    }

    fn arc_command(&mut self, base: Point, reader: &mut Reader) -> Option<()> {
        let radii = Point::new(reader.number()?, reader.number()?);
        let rotation = reader.number()?;
        let large = reader.flag()?;
        let sweep = reader.flag()?;
        let to = at(base, reader)?;
        let shape = Arc {
            radii,
            rotation,
            large,
            sweep,
        };
        let points = arc(self.current, shape, to);
        self.extend(points);
        Some(())
    }
}

fn at(base: Point, reader: &mut Reader) -> Option<Point> {
    let x = reader.number()?;
    Some(Point::new(base.x + x, base.y + reader.number()?))
}
