pub type Point = (f64, f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_corners(a: Point, b: Point) -> Self {
        let (x0, x1) = (a.0.min(b.0).floor(), a.0.max(b.0).ceil());
        let (y0, y1) = (a.1.min(b.1).floor(), a.1.max(b.1).ceil());
        Self::new(x0 as i32, y0 as i32, (x1 - x0) as u32, (y1 - y0) as u32)
    }

    pub const fn right(&self) -> i32 {
        self.x + self.w as i32
    }

    pub const fn bottom(&self) -> i32 {
        self.y + self.h as i32
    }

    pub const fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub const fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn area(&self) -> u64 {
        self.w as u64 * self.h as u64
    }

    pub fn covers(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    pub fn intersect(&self, other: &Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = self.right().min(other.right());
        let b = self.bottom().min(other.bottom());
        Rect::new(x, y, (r - x).max(0) as u32, (b - y).max(0) as u32)
    }

    pub fn scaled(&self, factor: f64) -> Rect {
        let s = |v: f64| (v * factor).round();
        Rect::new(
            s(self.x as f64) as i32,
            s(self.y as f64) as i32,
            s(self.w as f64) as u32,
            s(self.h as f64) as u32,
        )
    }

    pub fn relative_to(&self, origin: (i32, i32)) -> Rect {
        Rect::new(self.x - origin.0, self.y - origin.1, self.w, self.h)
    }

    pub fn grow_side(&self, side: Side, amount: i32) -> Rect {
        let (mut x0, mut y0, mut x1, mut y1) = (self.x, self.y, self.right(), self.bottom());
        match side {
            Side::Left => x0 -= amount,
            Side::Right => x1 += amount,
            Side::Top => y0 -= amount,
            Side::Bottom => y1 += amount,
        }
        Rect::new(
            x0.min(x1),
            y0.min(y1),
            (x1 - x0).max(0) as u32,
            (y1 - y0).max(0) as u32,
        )
    }

    pub fn centre(&self) -> (i32, i32) {
        (self.x + self.w as i32 / 2, self.y + self.h as i32 / 2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transform {
    #[default]
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    FlippedRotate90,
    FlippedRotate180,
    FlippedRotate270,
}

impl Transform {
    pub const fn swaps_axes(&self) -> bool {
        matches!(
            self,
            Self::Rotate90 | Self::Rotate270 | Self::FlippedRotate90 | Self::FlippedRotate270
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: Rect = Rect::new(100, 100, 50, 50);

    #[test]
    fn grow_side_pushes_only_that_edge_outward() {
        assert_eq!(R.grow_side(Side::Left, 5), Rect::new(95, 100, 55, 50));
        assert_eq!(R.grow_side(Side::Right, 5), Rect::new(100, 100, 55, 50));
        assert_eq!(R.grow_side(Side::Top, 5), Rect::new(100, 95, 50, 55));
        assert_eq!(R.grow_side(Side::Bottom, 5), Rect::new(100, 100, 50, 55));
    }

    #[test]
    fn grow_is_reversible_by_the_opposite_sign() {
        assert_eq!(R.grow_side(Side::Left, 5).grow_side(Side::Left, -5), R);
    }

    #[test]
    fn shrinking_past_zero_collapses_instead_of_inverting() {
        let collapsed = R.grow_side(Side::Left, -80);
        assert!(collapsed.is_empty());
        assert!(collapsed.x <= R.right());
    }

    #[test]
    fn from_corners_normalises_reverse_drag() {
        let r = Rect::from_corners((100.0, 80.0), (20.0, 10.0));
        assert_eq!(r, Rect::new(20, 10, 80, 70));
    }
}
