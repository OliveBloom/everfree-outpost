use std::prelude::v1::*;
use std::u16;

use physics::v3::{V3, scalar, Region, Align};

use Time;


pub const NUM_FRAMES: usize = 128;

pub struct Debug {
    pub frame_times: [u16; NUM_FRAMES],
    pub frame_intervals: [u16; NUM_FRAMES],
    pub cur_frame: usize,
    pub total_interval: usize,

    pub last_time: Time,

    pub ping: u32,
    pub pos: V3,
    pub day_time: u16,
    pub day_phase: u8,
}

impl Debug {
    pub fn new() -> Debug {
        Debug {
            frame_times: [0; NUM_FRAMES],
            frame_intervals: [0; NUM_FRAMES],
            cur_frame: 0,
            total_interval: 0,

            last_time: 0,

            ping: 0,
            pos: scalar(0),
            day_time: 0,
            day_phase: 0,
        }
    }

    /// Record the interval time for the previous frame, and increment `cur_frame` to refer to the
    /// actual frame we're currently rendering.
    pub fn record_interval(&mut self, now: Time) {
        let delta = now - self.last_time;
        let interval =
            if delta > u16::MAX as Time { u16::MAX }
            else { delta as u16 };

        self.total_interval -= self.frame_intervals[self.cur_frame] as usize;
        self.frame_intervals[self.cur_frame] = interval;
        self.total_interval += self.frame_intervals[self.cur_frame] as usize;

        self.cur_frame = (self.cur_frame + 1) % NUM_FRAMES;
        self.last_time = now;
    }

    pub fn record_frame_time(&mut self, time: Time) {
        self.frame_times[self.cur_frame] =
            if time > u16::MAX as Time { u16::MAX }
            else { time as u16 };
    }
}

