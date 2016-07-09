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


const TICK_MS: Time = 32;
const TICK_MASK: Time = TICK_MS - 1;

type ClientTime = Time;
type ServerTime = Time;
type Delta = Time;


const WHEEL_SIZE: usize = 64;

struct Wheel {
    /// List of client-server time deltas computed so far.
    wheel: [Time; WHEEL_SIZE],
    /// Index of the next `wheel` slot to be written.  This slot is considered empty/zero for
    /// purposes of computing `self.mean()`.
    index: usize,
    /// Accumulator for computing the mean.  Stores `mean * WHEEL_SIZE` (the sum of `wheel`).
    mean_acc: Time,
    /// Accumulator for computing the variance.  Stores `variance * WHEEL_SIZE * WHEEL_SIZE`.
    var_acc: Time,
}

impl Wheel {
    pub fn new(value: Time) -> Wheel {
        Wheel {
            wheel: [value; WHEEL_SIZE],
            index: 0,
            mean_acc: value * WHEEL_SIZE as Time,
            var_acc: 0,
        }
    }

    pub fn set(&mut self, val: Time) {
        let old = self.wheel[self.index];
        let new = val;

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

    pub fn advance(&mut self) {
        self.index = (self.index + 1) % WHEEL_SIZE;
    }

    pub fn set_advance(&mut self, val: Time) {
        self.set(val);
        self.advance();
    }

    pub fn mean(&self) -> Time {
        self.mean_acc / WHEEL_SIZE as Time
    }

    pub fn variance(&self) -> Time {
        self.var_acc / (WHEEL_SIZE * WHEEL_SIZE) as Time
    }

    pub fn stddev(&self) -> Time {
        round(sqrt(self.variance() as f64)) as Time
    }
}



pub struct Timing {
    delta: Wheel,
    ping: Wheel,
}

impl Timing {
    pub fn new() -> Timing {
        Timing {
            delta: Wheel::new(0),
            ping: Wheel::new(0),
        }
    }

    pub fn init(&mut self, client_send: ClientTime, client_recv: ClientTime, server: u16) {
        let server = server as Time;
        let delta = server - client_recv;
        let ping = client_recv - client_send;

        self.delta = Wheel::new(delta);
        self.ping = Wheel::new(ping);
    }

    pub fn record(&mut self, client_send: ClientTime, client_recv: ClientTime, server: u16) {
        let server = self.decode(client_recv, server);
        let delta = server - client_recv;
        let ping = client_recv - client_send;

        self.delta.set_advance(delta);
        self.ping.set_advance(ping);
    }

    pub fn record_delta(&mut self, client_recv: ClientTime, server: u16) {
        let server = self.decode(client_recv, server);
        let delta = server - client_recv;

        self.delta.set_advance(delta);
    }


    /// Decode a message time into a complete server time.
    pub fn decode(&self, client: ClientTime, server: u16) -> ServerTime {
        // Interpret `server` relative to a sliding window centered on our current approximation of
        // server time.
        let center: ServerTime = client + self.delta.mean();
        let offset = server.wrapping_sub(center as u16);
        center + offset as i16 as ServerTime
    }

    /// Get the server time corresponding to a given client time.
    pub fn convert(&self, client: ClientTime) -> ServerTime {
        client + self.delta.mean()
    }

    pub fn convert_confidence(&self, client: ClientTime, devs: i32) -> ServerTime {
        //println!("est. delta: {}, stddev: {}", self.delta.mean(), self.delta.stddev());
        client + self.delta.mean() + self.delta.stddev() * devs / 100
    }

    /// Predict the server time when a message will arrive, if sent at a given client time.
    pub fn predict(&self, client: ClientTime) -> ServerTime {
        self.convert(client) + self.ping.mean()
    }

    pub fn predict_confidence(&self, client: ClientTime, devs: i32) -> ServerTime {
        let ping = self.ping.mean() + self.ping.stddev() * devs / 100;
        self.convert_confidence(client, devs) + ping
    }
}
