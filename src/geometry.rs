#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_edges(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self::new(left, top, (right - left).max(0.0), (bottom - top).max(0.0))
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() && point.y >= self.y && point.y < self.bottom()
    }

    pub fn union(&self, other: Rect) -> Rect {
        Rect::from_edges(
            self.x.min(other.x),
            self.y.min(other.y),
            self.right().max(other.right()),
            self.bottom().max(other.bottom()),
        )
    }

    pub fn translated(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    pub fn intersect(&self, other: Rect) -> Rect {
        Rect::from_edges(
            self.x.max(other.x),
            self.y.max(other.y),
            self.right().min(other.right()),
            self.bottom().min(other.bottom()),
        )
    }

    pub fn inset(&self, amount: f32) -> Rect {
        self.inset_by(amount, amount)
    }

    pub fn inset_by(&self, horizontal: f32, vertical: f32) -> Rect {
        Rect::from_edges(
            self.x + horizontal,
            self.y + vertical,
            self.right() - horizontal,
            self.bottom() - vertical,
        )
    }

    pub fn split_left(&self, width: f32) -> (Rect, Rect) {
        let width = width.clamp(0.0, self.width);
        (
            Rect::new(self.x, self.y, width, self.height),
            Rect::new(self.x + width, self.y, self.width - width, self.height),
        )
    }

    pub fn split_right(&self, width: f32) -> (Rect, Rect) {
        let (rest, right) = self.split_left(self.width - width.clamp(0.0, self.width));
        (rest, right)
    }

    pub fn split_top(&self, height: f32) -> (Rect, Rect) {
        let height = height.clamp(0.0, self.height);
        (
            Rect::new(self.x, self.y, self.width, height),
            Rect::new(self.x, self.y + height, self.width, self.height - height),
        )
    }

    pub fn snapped(&self) -> Rect {
        Rect::from_edges(
            self.x.round(),
            self.y.round(),
            self.right().round(),
            self.bottom().round(),
        )
    }
}
