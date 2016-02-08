use std::hash::{Hash, Hasher, SipHasher};
use libserver_types::*;

pub struct Params {
    pub resolution: i32,
    pub offset: V2,
    pub magnitude: i32,
    pub seed: u64,
}

fn smooth(x: f64) -> f64 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x3 * x;
    let x5 = x4 * x;
    6.0 * x5 - 15.0 * x4 + 10.0 * x3
}

fn smooth_deriv(x: f64) -> f64 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x3 * x;
    30.0 * x4 - 60.0 * x3 + 30.0 * x2
}

fn hash_point(seed: u64, p: V2) -> u64 {
    let mut sip = SipHasher::new_with_keys(seed, 0x1234567);
    p.hash(&mut sip);
    sip.finish()
}

pub fn sample(p: &Params, pos: V2) -> i32 {
    let pos = pos - p.offset;
    let cell = pos.div_floor(scalar(p.resolution));

    let off = pos - cell * scalar(p.resolution);
    let x = off.x as f64 / p.resolution as f64;
    let y = off.y as f64 / p.resolution as f64;

    let wx = smooth(x);
    let wy = smooth(y);

    let cell_grad = |ox, oy| {
        let idx = hash_point(p.seed, cell + V2::new(ox, oy));
        GRADIENT_TABLE[idx as usize % GRADIENT_TABLE.len()]
    };

    let corner = |ox, oy| -> f64 {
        let (cx, cy) = cell_grad(ox, oy);
        (x - ox as f64) * cx + (y - oy as f64) * cy
    };

    let mut sum = 0.;

    {
        let mut go = |ox, oy, w: f64| {
            let cnr = corner(ox, oy);
            sum += cnr * w;
        };
        go(0, 0,  (1. - wx) * (1. - wy));
        go(1, 0,       (wx) * (1. - wy));
        go(0, 1,  (1. - wx) *      (wy));
        go(1, 1,       (wx) *      (wy));
    }

    (sum * p.magnitude as f64).round() as i32
}

pub fn gradient(p: &Params, pos: V2) -> V2 {
    let pos = pos - p.offset;
    let cell = pos.div_floor(scalar(p.resolution));

    let off = pos - cell * scalar(p.resolution);
    let x = off.x as f64 / p.resolution as f64;
    let y = off.y as f64 / p.resolution as f64;

    let wx = smooth(x);
    let wy = smooth(y);
    let dwx_dx = smooth_deriv(x);
    let dwy_dy = smooth_deriv(y);

    let cell_grad = |ox, oy| {
        let idx = hash_point(p.seed, cell + V2::new(ox, oy));
        GRADIENT_TABLE[idx as usize % GRADIENT_TABLE.len()]
    };

    let corner = |ox, oy| {
        let (cx, cy) = cell_grad(ox, oy);
        (x - ox as f64) * cx + (y - oy as f64) * cy
    };

    let mut dp_dx = 0.;
    let mut dp_dy = 0.;

    {
        let mut go = |ox, oy, w: f64, dw_dx: f64, dw_dy: f64| {
            let (cx, cy) = cell_grad(ox, oy);
            let cnr = corner(ox, oy);
            //sum += cnr * w;
            // `cx` / `cy` is the derivative of `corner`.
            dp_dx += cx * w + cnr * dw_dx;
            dp_dy += cy * w + cnr * dw_dy;
        };
        go(0, 0,  (1. - wx) * (1. - wy),  -dwx_dx * (1. - wy),  (1. - wx) * -dwy_dy);
        go(1, 0,       (wx) * (1. - wy),   dwx_dx * (1. - wy),       (wx) * -dwy_dy);
        go(0, 1,  (1. - wx) *      (wy),  -dwx_dx *      (wy),  (1. - wx) *  dwy_dy);
        go(1, 1,       (wx) *      (wy),   dwx_dx *      (wy),       (wx) *  dwy_dy);
    }

    V2::new((dp_dx * p.magnitude as f64).round() as i32,
            (dp_dy * p.magnitude as f64).round() as i32)
}


// AUTO-GENERATED by src/gen/tables/gen_perlin_gradients.py at Mon Feb  8 07:02:53 2016
static GRADIENT_TABLE: [(f64, f64); 64] = [
    (1.000000000000, 0.000000000000),
    (0.995184726672, 0.098017140330),
    (0.980785280403, 0.195090322016),
    (0.956940335732, 0.290284677254),
    (0.923879532511, 0.382683432365),
    (0.881921264348, 0.471396736826),
    (0.831469612303, 0.555570233020),
    (0.773010453363, 0.634393284164),
    (0.707106781187, 0.707106781187),
    (0.634393284164, 0.773010453363),
    (0.555570233020, 0.831469612303),
    (0.471396736826, 0.881921264348),
    (0.382683432365, 0.923879532511),
    (0.290284677254, 0.956940335732),
    (0.195090322016, 0.980785280403),
    (0.098017140330, 0.995184726672),
    (0.000000000000, 1.000000000000),
    (-0.098017140330, 0.995184726672),
    (-0.195090322016, 0.980785280403),
    (-0.290284677254, 0.956940335732),
    (-0.382683432365, 0.923879532511),
    (-0.471396736826, 0.881921264348),
    (-0.555570233020, 0.831469612303),
    (-0.634393284164, 0.773010453363),
    (-0.707106781187, 0.707106781187),
    (-0.773010453363, 0.634393284164),
    (-0.831469612303, 0.555570233020),
    (-0.881921264348, 0.471396736826),
    (-0.923879532511, 0.382683432365),
    (-0.956940335732, 0.290284677254),
    (-0.980785280403, 0.195090322016),
    (-0.995184726672, 0.098017140330),
    (-1.000000000000, 0.000000000000),
    (-0.995184726672, -0.098017140330),
    (-0.980785280403, -0.195090322016),
    (-0.956940335732, -0.290284677254),
    (-0.923879532511, -0.382683432365),
    (-0.881921264348, -0.471396736826),
    (-0.831469612303, -0.555570233020),
    (-0.773010453363, -0.634393284164),
    (-0.707106781187, -0.707106781187),
    (-0.634393284164, -0.773010453363),
    (-0.555570233020, -0.831469612303),
    (-0.471396736826, -0.881921264348),
    (-0.382683432365, -0.923879532511),
    (-0.290284677254, -0.956940335732),
    (-0.195090322016, -0.980785280403),
    (-0.098017140330, -0.995184726672),
    (-0.000000000000, -1.000000000000),
    (0.098017140330, -0.995184726672),
    (0.195090322016, -0.980785280403),
    (0.290284677254, -0.956940335732),
    (0.382683432365, -0.923879532511),
    (0.471396736826, -0.881921264348),
    (0.555570233020, -0.831469612303),
    (0.634393284164, -0.773010453363),
    (0.707106781187, -0.707106781187),
    (0.773010453363, -0.634393284164),
    (0.831469612303, -0.555570233020),
    (0.881921264348, -0.471396736826),
    (0.923879532511, -0.382683432365),
    (0.956940335732, -0.290284677254),
    (0.980785280403, -0.195090322016),
    (0.995184726672, -0.098017140330),
];
