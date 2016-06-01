#![crate_name = "common_movement"]
#![no_std]

#[cfg(asmjs)] #[macro_use] extern crate fakestd as std;
#[cfg(not(asmjs))] #[macro_use] extern crate std;
use std::prelude::v1::*;

// TODO: currently this is the way to get the asm.js log macros
#[cfg(asmjs)] #[macro_use] extern crate asmrt;
#[cfg(not(asmjs))] #[macro_use] extern crate log;

#[macro_use] extern crate bitflags;

extern crate physics;

use physics::ShapeSource;
use physics::v3::{V3, Region, scalar};


bitflags! {
    pub flags InputBits: u16 {
        const INPUT_LEFT =      0x0001,
        const INPUT_RIGHT =     0x0002,
        const INPUT_UP =        0x0004,
        const INPUT_DOWN =      0x0008,
        const INPUT_RUN =       0x0010,
        const INPUT_HOLD =      0x0020,

        const INPUT_DIR_MASK =  INPUT_LEFT.bits |
                                INPUT_RIGHT.bits |
                                INPUT_UP.bits |
                                INPUT_DOWN.bits,
    }
}

impl InputBits {
    pub fn to_direction(&self) -> Option<V3> {
        if (*self & INPUT_DIR_MASK) == InputBits::empty() {
            return None
        }

        let x =
            if self.contains(INPUT_LEFT) { -1 } else { 0 } +
            if self.contains(INPUT_RIGHT) { 1 } else { 0 };
        let y =
            if self.contains(INPUT_UP) { -1 } else { 0 } +
            if self.contains(INPUT_DOWN) { 1 } else { 0 };
        Some(V3::new(x, y, 0))
    }

    pub fn to_speed(&self) -> u8 {
        if self.contains(INPUT_HOLD) ||
           (*self & INPUT_DIR_MASK) == InputBits::empty() { 0 }
        else if self.contains(INPUT_RUN) { 3 }
        else { 1 }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    Walk,
    //Fly,
    Busy,
}


pub trait Time: Copy {
    fn interval(self, prev: Self) -> i32;
    fn offset(self, offset: i32) -> Self;
}

impl Time for i32 {
    fn interval(self, prev: i32) -> i32 {
        self - prev
    }

    fn offset(self, offset: i32) -> i32 {
        self + offset
    }
}

impl Time for i64 {
    fn interval(self, prev: i64) -> i32 {
        (self - prev) as i32
    }

    fn offset(self, offset: i32) -> i64 {
        self + offset as i64
    }
}


pub trait Entity {
    fn activity(&self) -> Activity;
    fn facing(&self) -> V3;
    fn velocity(&self) -> V3;

    type Time: Time;
    fn pos(&self, now: Self::Time) -> V3;

    fn start_motion(&mut self, now: Self::Time, pos: V3, facing: V3, speed: u8, velocity: V3);
    fn end_motion(&mut self, now: Self::Time);
}


pub struct MovingEntity {
    input: InputBits,
    force_update: bool,
}

impl MovingEntity {
    pub fn new() -> MovingEntity {
        MovingEntity {
            input: InputBits::empty(),
            force_update: false,
        }
    }

    pub fn set_input(&mut self, input: InputBits) {
        if input != self.input {
            self.input = input;
            self.force_update = true;
        }
    }

    pub fn force_update(&mut self) {
        self.force_update = true;
    }

    pub fn update<S, E>(&mut self, e: &mut E, s: &S, now: E::Time, next: E::Time)
            where S: ShapeSource, E: Entity {
        if e.activity() == Activity::Busy {
            return;
        }

        let pos = e.pos(now);
        let size = V3::new(32, 32, 48);
        let mut collider = physics::walk2::Collider::new(s, Region::new(pos, pos + size));


        let facing = self.input.to_direction().unwrap_or(e.facing());
        let speed = self.input.to_speed();
        let target_velocity = facing * scalar(speed as i32 * 50);

        let velocity = collider.calc_velocity(target_velocity);
        let started = velocity != e.velocity() ||
                      self.force_update;
        if started {
            self.force_update = false;
            e.start_motion(now, pos, facing, speed, velocity);
        }


        let next_pos = e.pos(next);
        let step_ms = next.interval(now);
        let (_step, dur) = collider.walk(next_pos - pos, step_ms);
        let ended = dur != step_ms;
        if ended {
            e.end_motion(now.offset(dur));
        }
    }
}
