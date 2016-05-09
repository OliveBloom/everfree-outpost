use types::*;


pub const TICK_MS: Time = 32;

pub fn next_tick(t: Time) -> Time {
    // This isn't just `ceil`.  We need `next_tick(n * TICK_MS) == (n + 1) * TICK_MS`.
    prev_tick(t + TICK_MS)
}

pub fn prev_tick(t: Time) -> Time {
    // This, however, is just floor.
    t & !(TICK_MS - 1)
}

