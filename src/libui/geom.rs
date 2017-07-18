use std::ops::Add;

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

impl Rect {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
        Rect {
            min: Point { x: x0, y: y0 },
            max: Point { x: x1, y: y1 },
        }
    }

    pub fn contains(&self, pos: Point) -> bool {
        self.min.x <= pos.x && pos.x < self.max.x &&
        self.min.y <= pos.y && pos.y < self.max.y
    }
}
