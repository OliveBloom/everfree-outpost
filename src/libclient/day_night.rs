use std::prelude::v1::*;

use Time;
use data::Data;


pub const DAY_NIGHT_CYCLE: u16 = 24000;

pub struct DayNight {
    base_time: Time,
    cycle_ms: Time,
    factor: Time,
}

impl DayNight {
    pub fn new() -> DayNight {
        DayNight {
            base_time: 0,
            cycle_ms: 24000,
            factor: 1,
        }
    }

    pub fn init(&mut self, base_time: Time, cycle_ms: Time) {
        self.base_time = base_time;
        self.cycle_ms = cycle_ms;
        self.factor = cycle_ms / DAY_NIGHT_CYCLE as i32;
    }

    /// Returns the time of day for the given timestamp.  Time of day is always in the range
    /// 0 .. DAY_NIGHT_CYCLE.
    pub fn time_of_day(&self, now: Time) -> u16 {
        let delta = now - self.base_time;
        let ms_time = delta % self.cycle_ms;
        let ms_time = if ms_time < 0 { ms_time + self.cycle_ms } else { ms_time };
        // The formula we want is:
        //    ms_time * DAY_NIGHT_CYCLE / self.cycle_ms
        // But that may require 64-bit arithmetic.  So instead we use this alternative formula,
        // which is just as good as long as cycle_ms is a multiple of DAY_NIGHT_CYCLE.
        (ms_time / self.factor) as u16
    }

    pub fn phase_delta(&self, data: &Data, time_of_day: u16) -> (u8, u16) {
        for (i, p) in data.day_night_phases().iter().enumerate() {
            if time_of_day < p.end_time {
                return (i as u8, time_of_day - p.start_time);
            }
        }
        panic!("no day/night phase covers time {}", time_of_day);
    }

    pub fn ambient_light_for_phase(&self,
                                   data: &Data,
                                   phase: u8,
                                   delta: u16) -> (u8, u8, u8, u8) {
        let p = data.day_night_phase(phase);

        let delta = delta as u32;
        let dur = (p.end_time - p.start_time) as u32;

        let idx0 = p.start_color as u32;
        let idx1 = p.end_color as u32;
        // Index of the color stop immediately before `delta`.
        let idx = idx0 + (idx1 - idx0) * delta / dur;

        // Timestamps of stops `idx` and `idx + 1`.
        let t0 = dur * (idx - idx0) / (idx1 - idx0);
        let t1 = dur * (idx - idx0 + 1) / (idx1 - idx0);

        // idx0        idx   idx+1       idx1       <- index
        //  |--- ... ---|--*---|--- ... ---|
        //  0           t0 |   t1         dur       <- time
        //                 |
        //                delta

        let blend = |a: u8, b: u8| -> u8 {
            let mix = a as u32 * (t1 - delta) +
                      b as u32 * (delta - t0);
            (mix / (t1 - t0)) as u8
        };

        let (r1, g1, b1) = data.day_night_colors()[idx as usize];
        let (r2, g2, b2) = data.day_night_colors()[idx as usize + 1];

        let r = blend(r1, r2);
        let g = blend(g1, g2);
        let b = blend(b1, b2);

        // Calculate grayscale intensity from RGB
        let i = ((2126 * r as u32 +
                  7152 * g as u32 +
                   722 * b as u32) / 10000) as u8;

        (r,g,b,i)
    }

    pub fn ambient_light(&self, data: &Data, now: Time) -> (u8, u8, u8, u8) {
        let tod = self.time_of_day(now);
        let (phase, delta) = self.phase_delta(data, tod);
        self.ambient_light_for_phase(data, phase, delta)
    }
}
