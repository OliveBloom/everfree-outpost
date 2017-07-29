use outpost_ui::geom::{Point, Rect};
use physics::v3::{V2, scalar, Region};


pub fn from_v2(v: V2) -> Point {
    Point {
        x: v.x,
        y: v.y,
    }
}

pub fn to_v2(p: Point) -> V2 {
    V2::new(p.x, p.y)
}

pub fn from_region2(r: Region<V2>) -> Rect {
    Rect {
        min: from_v2(r.min),
        max: from_v2(r.max),
    }
}

pub fn to_region2(r: Rect) -> Region<V2> {
    Region {
        min: to_v2(r.min),
        max: to_v2(r.max),
    }
}
