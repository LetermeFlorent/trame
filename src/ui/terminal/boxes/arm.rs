use super::lines::{DOUBLE_SECOND_STROKE, DOUBLE_WIDTH, Span, Weight};

enum Band {
    None,
    Single { start: f32, thickness: f32 },
    Double { start: f32 },
}

pub(super) struct Arm {
    pub(super) weight: Weight,
    pub(super) high: bool,
    pub(super) opposite: Weight,
    pub(super) before: Weight,
    pub(super) after: Weight,
}

impl Arm {
    pub(super) fn strokes(&self, along: Span, across: Span, light: f32) -> Vec<[f32; 4]> {
        match self.weight {
            Weight::None => Vec::new(),
            Weight::Double => {
                let start = across.centered(DOUBLE_WIDTH * light);
                vec![
                    self.stroke(along, start, light, self.double_reach(true, along, light)),
                    self.stroke(
                        along,
                        start + DOUBLE_SECOND_STROKE * light,
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
                start: along.centered(DOUBLE_WIDTH * light),
            };
        }
        let thickness = self
            .before
            .thickness(light)
            .max(self.after.thickness(light));
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
                self.reach_gap(start, light)
            }
            Band::Double { start } => self.reach_past_band(start, light),
        }
    }

    fn double_reach(&self, before_side: bool, along: Span, light: f32) -> f32 {
        let near = if before_side { self.before } else { self.after };
        match self.band(along, light) {
            Band::None => along.center(),
            Band::Single { start, thickness } => self.pick(start + thickness, start),
            Band::Double { start } if near != Weight::None => self.reach_gap(start, light),
            Band::Double { start } => self.reach_past_band(start, light),
        }
    }

    fn reach_gap(&self, start: f32, light: f32) -> f32 {
        self.pick(start + light, start + DOUBLE_SECOND_STROKE * light)
    }

    fn reach_past_band(&self, start: f32, light: f32) -> f32 {
        self.pick(start + DOUBLE_WIDTH * light, start)
    }

    fn pick(&self, low: f32, high: f32) -> f32 {
        if self.high { high } else { low }
    }
}
