use std::cmp;
use std::i32;

use Time;


pub struct Gauge {
    last_val: i32,
    last_time: Time,

    /// The delta (= now - last_time) at which the value will hit `min` or `max`.
    max_delta: Time,

    /// The numerator of the rate of change.
    rate_numer: i16,
    /// The denominator of the rate of change.
    // Note: rate_denom is measured in seconds, but most code works in millisecnds, so most places
    // use `1000 * rate_denom`.
    rate_denom: u16,
    min: i32,
    max: i32,
}

impl Gauge {
    pub fn new(cur: i32, rate: (i16, u16), now: Time, min: i32, max: i32) -> Gauge {
        let mut g = Gauge {
            last_val: cur,
            last_time: now,
            max_delta: 0,
            rate_numer: rate.0,
            rate_denom: rate.1,
            min: min,
            max: max,
        };
        g.max_delta = g.calc_max_delta();
        g
    }

    fn calc_max_delta(&self) -> Time {
        let d = 1000 * self.rate_denom as i32;
        if self.rate_numer > 0 {
            let n = self.rate_numer as i32;
            ((self.max - self.last_val) * d + n - 1) / n
        } else if self.rate_numer < 0 {
            let n = -self.rate_numer as i32;
            ((self.last_val - self.min) * d + n - 1) / n
        } else {    // self.rate == 0
            i32::MAX
        }
    }

    pub fn get(&self, time: Time) -> i32 {
        let delta = time - self.last_time;

        if delta >= self.max_delta {
            if self.rate_numer > 0 {
                self.max
            } else {
                self.min
            }
        } else {
            let n = self.rate_numer as i32;
            let d = 1000 * self.rate_denom as i32;
            self.last_val + delta * n / d
        }
    }

    pub fn set(&mut self, val: i32, time: Time) -> i32 {
        let val = cmp::max(self.min, cmp::min(val, self.max));
        self.last_val = val;
        self.last_time = time;
        self.max_delta = self.calc_max_delta();
        val
    }

    pub fn adjust(&mut self, adj: i32, time: Time) -> i32 {
        let val = self.get(time) + adj;
        self.set(val, time)
    }

    pub fn set_rate(&mut self, numer: i16, denom: u16, time: Time) {
        // Update so that last_time = time.
        let val = self.get(time);
        self.set(val, time);

        self.rate_numer = numer;
        self.rate_denom = denom;
        self.max_delta = self.calc_max_delta();
    }


    pub fn min(&self) -> i32 {
        self.min
    }

    pub fn max(&self) -> i32 {
        self.max
    }

    pub fn rate(&self) -> (i16, u16) {
        (self.rate_numer, self.rate_denom)
    }
}
