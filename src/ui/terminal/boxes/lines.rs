use crate::color::Rgba;
use crate::geometry::Rect;
use crate::render::Scene;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Weight {
    None,
    Light,
    Heavy,
    Double,
}

impl Weight {
    fn from_nibble(code: u16, shift: u32) -> Self {
        match (code >> shift) & 0xF {
            1 => Self::Light,
            2 => Self::Heavy,
            3 => Self::Double,
            _ => Self::None,
        }
    }

    fn thickness(self, light: f32) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Light => light,
            Self::Heavy => light * 2.0,
            Self::Double => light * 3.0,
        }
    }
}

#[derive(Clone, Copy)]
struct Span {
    lo: f32,
    size: f32,
}

impl Span {
    fn centered(self, thickness: f32) -> f32 {
        self.lo + ((self.size - thickness) / 2.0).floor()
    }

    fn center(self) -> f32 {
        (self.lo + self.size / 2.0).round()
    }

    fn hi(self) -> f32 {
        self.lo + self.size
    }
}

enum Band {
    None,
    Single { start: f32, thickness: f32 },
    Double { start: f32 },
}

struct Arm {
    weight: Weight,
    high: bool,
    opposite: Weight,
    before: Weight,
    after: Weight,
}

impl Arm {
    fn strokes(&self, along: Span, across: Span, light: f32) -> Vec<[f32; 4]> {
        match self.weight {
            Weight::None => Vec::new(),
            Weight::Double => {
                let start = across.centered(3.0 * light);
                vec![
                    self.stroke(along, start, light, self.double_reach(true, along, light)),
                    self.stroke(
                        along,
                        start + 2.0 * light,
                        light,
                        self.double_reach(false, along, light),
                    ),
                ]
            }
            weight => {
                let thickness = weight.thickness(light);
                let start = across.centered(thickness);
                let reach = self.single_reach(along, thickness, light);
                vec![self.stroke(along, start, thickness, reach)]
            }
        }
    }

    fn stroke(&self, along: Span, start: f32, thickness: f32, reach: f32) -> [f32; 4] {
        let (from, to) = if self.high {
            (reach, along.hi())
        } else {
            (along.lo, reach)
        };
        [from, to, start, start + thickness]
    }

    fn band(&self, along: Span, light: f32) -> Band {
        if self.before == Weight::Double || self.after == Weight::Double {
            return Band::Double {
                start: along.centered(3.0 * light),
            };
        }
        let thickness = self.before.thickness(light).max(self.after.thickness(light));
        if thickness == 0.0 {
            return Band::None;
        }
        Band::Single {
            start: along.centered(thickness),
            thickness,
        }
    }

    fn single_reach(&self, along: Span, thickness: f32, light: f32) -> f32 {
        if self.opposite != Weight::None {
            return along.center();
        }
        match self.band(along, light) {
            Band::None => {
                let start = along.centered(thickness);
                self.pick(start + thickness, start)
            }
            Band::Single { start, thickness } => self.pick(start + thickness, start),
            Band::Double { start }
                if self.before == Weight::Double && self.after == Weight::Double =>
            {
                self.pick(start + light, start + 2.0 * light)
            }
            Band::Double { start } => self.pick(start + 3.0 * light, start),
        }
    }

    fn double_reach(&self, before_side: bool, along: Span, light: f32) -> f32 {
        let near = if before_side { self.before } else { self.after };
        match self.band(along, light) {
            Band::None => along.center(),
            Band::Single { start, thickness } => self.pick(start + thickness, start),
            Band::Double { start } if near != Weight::None => {
                self.pick(start + light, start + 2.0 * light)
            }
            Band::Double { start } => self.pick(start + 3.0 * light, start),
        }
    }

    fn pick(&self, low: f32, high: f32) -> f32 {
        if self.high { high } else { low }
    }
}

pub fn draw_lines(scene: &mut Scene, code: u16, rect: Rect, color: Rgba, light: f32) {
    let [up, right, down, left] = [12, 8, 4, 0].map(|shift| Weight::from_nibble(code, shift));
    let xs = Span {
        lo: rect.x,
        size: rect.width,
    };
    let ys = Span {
        lo: rect.y,
        size: rect.height,
    };
    let arm = |weight, high, opposite, before, after| Arm {
        weight,
        high,
        opposite,
        before,
        after,
    };
    let arms = [
        (arm(left, false, right, up, down), false),
        (arm(right, true, left, up, down), false),
        (arm(up, false, down, left, right), true),
        (arm(down, true, up, left, right), true),
    ];
    for (arm, vertical) in arms {
        let (along, across) = if vertical { (ys, xs) } else { (xs, ys) };
        for [from, to, start, end] in arm.strokes(along, across, light) {
            let stroke = if vertical {
                Rect::from_edges(start, from, end, to)
            } else {
                Rect::from_edges(from, start, to, end)
            };
            scene.fill(stroke, color, 0.0);
        }
    }
}
