use std::ops::{Add, Sub};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

pub const ORIGIN: Point = Point { x: 0, y: 0 };


impl Add<Point> for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub<Point> for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}


impl Rect {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
        Rect {
            min: Point { x: x0, y: y0 },
            max: Point { x: x1, y: y1 },
        }
    }

    pub fn sized(size: Point) -> Rect {
        Rect {
            min: Point { x: 0, y: 0 },
            max: size,
        }
    }

    pub fn contains(&self, pos: Point) -> bool {
        self.min.x <= pos.x && pos.x < self.max.x &&
        self.min.y <= pos.y && pos.y < self.max.y
    }

    pub fn center(&self, inner: Rect) -> Rect {
        let x = ((self.max.x - self.min.x) - (inner.max.x - inner.min.x)) / 2;
        let y = ((self.max.y - self.min.y) - (inner.max.y - inner.min.y)) / 2;
        let offset = self.min + Point { x: x, y: y };
        Rect {
            min: inner.min + offset,
            max: inner.max + offset,
        }
    }

    pub fn inset(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
        Rect {
            min: self.min + Point { x: x0, y: y0 },
            max: self.max + Point { x: x0, y: y0 },
        }
    }

    pub fn size(&self) -> Point {
        self.max - self.min
    }
}

impl Add<Point> for Rect {
    type Output = Rect;
    fn add(self, other: Point) -> Rect {
        Rect {
            min: self.min + other,
            max: self.max + other,
        }
    }
}

