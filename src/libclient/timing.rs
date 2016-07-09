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
    /// Accumulator for computing the mean.  Stores `mean * WHEEL_SIZE` (the sum of `wheel`).
    mean_acc: Delta,
    /// Accumulator for computing the variance.  Stores `variance * WHEEL_SIZE * WHEEL_SIZE`.
    var_acc: Delta,

    /// Server timestamp of the tick currently being processed.
    tick: ServerTime,
    /// Client timestamp of the first packet received for this tick.
    first: ClientTime,
    /// Client timestamp of the last packet received for this tick.
    last: ClientTime,
}

impl TimingRecv {
    pub fn new() -> TimingRecv {
        TimingRecv {
            wheel: [0; WHEEL_SIZE],
            index: 0,
            mean_acc: 0,
            var_acc: 0,

            tick: 0,
            first: 0,
            last: 0,
        }
    }


    fn mean(&self) -> Delta {
        self.mean_acc / WHEEL_SIZE as Delta
    }

    fn variance(&self) -> Delta {
        self.var_acc / (WHEEL_SIZE * WHEEL_SIZE) as Delta
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
        println!("est. delta: {}, stddev: {}", self.mean(), self.stddev());
        client + self.mean() + self.stddev() * devs / 100
    }


    pub fn init(&mut self, client: Time, server: u16) {
        self.tick = (server as Time) & !TICK_MASK;
        self.first = client;
        self.last = client;

        let delta = self.calc_delta();
        for slot in self.wheel.iter_mut() {
            *slot = delta;
        }
        self.mean_acc = delta * WHEEL_SIZE as Delta;
        self.var_acc = 0;
    }

    pub fn record(&mut self, client: Time, server: u16) {
        let msg = server;
        let server = self.decode(client, server);
        println!(" - recording: msg = {}, client = {}, server = {}, delta = {}",
                 msg, client, server, server - client);

        let tick = server & !TICK_MASK;
        if tick != self.tick {
            self.index = (self.index + 1) % WHEEL_SIZE;
            self.tick = tick;
            self.first = client;
            self.last = client;
        } else {
            self.first = cmp::min(self.first, client);
            self.last = cmp::max(self.last, client);
        }
        self.update_acc();
    }

    /// Replace the current slot in `wheel` with the result of `calc_delta()`, and update
    /// accumulators.
    fn update_acc(&mut self) {
        let old = self.wheel[self.index];
        let new = self.calc_delta();

        let old_mean = self.mean_acc;
        let new_mean = self.mean_acc + new - old;

        const N: Delta = WHEEL_SIZE as Delta;
        let old_var = self.var_acc;
        let adj = (new - old) * (N * new - new_mean + N * old - old_mean);
        let new_var = self.var_acc + adj;

        self.wheel[self.index] = new;
        self.mean_acc = new_mean;
        self.var_acc = new_var;
    }

    /// Calculate the delta for the tick currently being measured.
    fn calc_delta(&self) -> Delta {
        // Assume messages are generally sent about 1/4 of the way through the tick.  This
        // adjustment somewhat reduces the bias due to number of messages sent in the tick.
        let dur = self.last - self.first;
        let extra = TICK_MS - dur;
        // Client timestamp corresponding to the start of the tick.
        let client = self.first - extra * 1 / 4;
        self.tick - client
    }
}

pub type Timing = TimingRecv;
