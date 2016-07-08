//! Note on time systems:
//!
//! For the purposes of this module, there are two relevant systems of time.  Client time starts
//! counting from zero at the moment the client was started.  Server time starts counting from the
//! timestamp of the first received packet at the moment that packet arrives.  Since packet
//! timestamps are 16 bits, that means server time starts roughly 0 - 65 seconds before client time
//! does.  One goal of this module is to compute a good approximation of this offset, so that we
//! can obtain the server time corresponding to any client time.
//!
//! Note that "server time" does not actually correspond to the precise time on the server.
//! Rather, it gives the time on the server plus the one-way travel time from server to client,
//! which is the timestamp of the newest state change the client can observe.  This means that
//! server time can drift due to changes in network conditions.

use std::prelude::v1::*;
use std::cmp;

use Time;
use util::{sqrt, round};


const WHEEL_SIZE: usize = 64;
const TICK_MS: Time = 32;
const TICK_MASK: Time = TICK_MS - 1;

type ClientTime = Time;
type ServerTime = Time;
type Delta = Time;

pub struct TimingRecv {
    /// List of client-server time deltas computed so far.
    wheel: [Delta; WHEEL_SIZE],
    /// Index of the next `wheel` slot to be written.  This slot is considered empty/zero for
    /// purposes of computing `self.mean()`.
    index: usize,
    /// Accumulator for deltas in `wheel`, used for computing mean and stddev.
    sum_acc: Delta,
    /// Accumulator for squares of deltas in `wheel`, used for computing stddev.
    ///
    /// This actually stores the sum divided by `WHEEL_SIZE * 4`, since deltas may range up to 2^16
    /// and we need to avoid having the sum overflow 2^31.
    sum2_acc: Delta,

    /// Server timestamp of the tick currently being processed.
    tick: ServerTime,
    /// Client timestamp of the first packet received for this tick.
    first: ClientTime,
    /// Client timestamp of the last packet received for this tick.
    last: ClientTime,
}

/// Carefully compute `x * x / WHEEL_SIZE / 4` to avoid overflow.
fn smart_square(x: Delta) -> Delta {
    // Compute `x/2 * x/2`, with one division rounding down and the other rounding up.  This
    // provides good precision.
    let a = (x + 0) / 2;
    let b = (x + 1) / 2;

    let d = WHEEL_SIZE as Delta;
    (a * b + (d / 2)) / d
}

impl TimingRecv {
    fn new() -> TimingRecv {
        TimingRecv {
            wheel: [0; WHEEL_SIZE],
            index: 0,
            sum_acc: 0,
            sum2_acc: 0,

            tick: 0,
            first: 0,
            last: 0,
        }
    }


    fn cur_delta(&self) -> Delta {
        // Assume messages are generally sent about 1/4 of the way through the tick.  This
        // adjustment somewhat reduces the bias due to number of messages sent in the tick.
        let dur = self.last - self.first;
        let extra = TICK_MS - dur;
        self.last + extra * 3 / 4
    }

    fn cur_sum(&self) -> Delta {
        self.sum_acc + self.cur_delta()
    }

    fn cur_sum2(&self) -> Delta {
        self.sum2_acc + smart_square(self.cur_delta())
    }

    fn mean(&self) -> Delta {
        self.cur_sum() / (WHEEL_SIZE as Delta)
    }

    fn variance(&self) -> Delta {
        let a = self.cur_sum2();
        let b = smart_square(self.cur_sum());
        (a - b) * 4
    }

    fn stddev(&self) -> Delta {
        round(sqrt(self.variance() as f64)) as Delta
    }

    pub fn decode(&self, client: ClientTime, server: u16) -> ServerTime {
        let server_now = client + self.mean();
        let offset = server.wrapping_sub(server_now as u16);
        server_now + offset as i16 as ServerTime
    }

    pub fn convert(&self, client: ClientTime) -> ServerTime {
        client + self.mean()
    }

    pub fn convert_confidence(&self, client: ClientTime, devs: i32) -> ServerTime {
        client + self.mean() + self.stddev() * devs / 100
    }


    fn init(&mut self, client: Time, server: u16) {
        self.tick = (server as Time) & !TICK_MASK;
        self.first = client;
        self.last = client;
    }

    fn record(&mut self, client: Time, server: u16) {
        let server = self.decode(client, server);

        let tick = server & !TICK_MASK;
        if tick != self.tick {
            self.commit_tick();
            self.tick = tick;
            self.first = client;
            self.last = client;
        } else {
            self.first = cmp::min(self.first, client);
            self.last = cmp::max(self.last, client);
        }
    }

    fn commit_tick(&mut self) {
        let old = self.wheel[(self.index + 1) % WHEEL_SIZE];
        let new = self.cur_delta();

        self.sum_acc += new - old;
        self.sum2_acc += smart_square(new) - smart_square(old);

        self.wheel[self.index] = new;
        self.index = (self.index + 1) % WHEEL_SIZE;
    }
}
