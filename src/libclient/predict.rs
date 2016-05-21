use std::prelude::v1::*;
use std::collections::VecDeque;
use std::u16;

use physics::v3::{V3, Vn, scalar};
use physics::{self, ShapeSource};

use Time;
use data::Data;
use entity::{Motion, Update};


struct Input {
    time: Time,
    dir: V3,
}

/// The `Predictor` answers queries about the player's future position, based on inputs that have
/// been sent to the server but (possibly) not processed yet.
///
/// The timing module has a notion of "latest visible time", the server timestamp of the most
/// recent event the client could have seen (in other words, the current time minus the
/// server-to-client latency).  Most client code displays the state of the world as of the latest
/// visible time.  The predictor, on the other hand, tries to guess what messages will arrive
/// with timestamps between the latest visible time and some future (server) time, so that it can
/// report the state of the world as of that future time.
pub struct Predictor {
    /// Inputs that have been sent to the server but not acknowledged.
    pending_inputs: VecDeque<Input>,

    /// Last canonical motion received from the server.
    canon_motion: Motion,

    /// The current motion as of the future time.
    current_motion: Motion,

    /// Timestamp of the last update.
    current_time: Time,
}

// This must match the server tick.
pub const TICK_MS: i32 = 32;

impl Predictor {
    pub fn new() -> Predictor {
        Predictor {
            pending_inputs: VecDeque::new(),
            canon_motion: Motion::new(),
            current_motion: Motion::new(),
            // Initialize to a time prior to any time reported by timing.js
            current_time: -65536,
        }
    }

    pub fn canonical_motion_update(&mut self, update: Update) {
        self.canon_motion.apply(update);
        self.current_motion = self.canon_motion.clone();
        // Rewind the current time to the time of the update, so that inputs will be replayed.
        self.current_time = update.when();
    }

    pub fn input<S>(&mut self, time: Time, dir: V3, shape: &S, data: &Data)
            where S: ShapeSource {
        let input = Input { time: time, dir: dir };
        /*
        play_input(&mut self.motion,
                   &mut self.cur_dir,
                   &input,
                   shape,
                   data);
                   */
        self.pending_inputs.push_back(input);
    }

    pub fn update<S>(&mut self, future: Time, shape: &S, data: &Data)
            where S: ShapeSource {
        let cur_tick = future & !(TICK_MS - 1);
        while self.current_time < future {
            // TODO: play an input
        }
        /*
        if self.stale {
            // Replay all inputs.
            self.cur_dir = self.last_dir;
            for input in self.inputs.iter() {
                play_input(&mut self.motion,
                           &mut self.cur_dir,
                           input,
                           shape,
                           data);
            }
            self.stale = false;
        }

        while self.motion.end_time < now {
            self.motion = predict(shape,
                                  data,
                                  self.motion.end_pos,
                                  self.motion.end_time,
                                  self.motion.anim_id,
                                  self.cur_dir);
        }
        */
    }

    pub fn motion(&self) -> &Motion {
        &self.current_motion
    }
}

/*
fn play_input<S>(motion: &mut Motion,
                 dir: &mut V3,
                 input: &Input,
                 shape: &S,
                 data: &Data)
        where S: ShapeSource {
    // Play forward until the time of the input event.
    while motion.end_time < input.time {
        *motion = predict(shape,
                          data,
                          motion.end_pos,
                          motion.end_time,
                          motion.anim_id,
                          *dir);
    }

    // Play the input event.
    *dir = input.dir;
    *motion = predict(shape,
                      data,
                      motion.pos(input.time),
                      input.time,
                      motion.anim_id,
                      *dir);
}
*/

/*
fn predict<S: ShapeSource>(shape: &S,
                           data: &Data,
                           start_pos: V3,
                           start_time: Time,
                           old_anim: u16,
                           target_velocity: V3) -> Motion {
    // TODO: hardcoded constant
    let size = V3::new(32, 32, 64);
    let (mut end_pos, mut dur) = physics::collide(shape, start_pos, size, target_velocity);


    // NB: keep this in sync with server/physics.rs
    const DURATION_MAX: u16 = u16::MAX;
    if dur > DURATION_MAX as i32 {
        let offset = end_pos - start_pos;
        end_pos = start_pos + offset * scalar(DURATION_MAX as i32) / scalar(dur);
        dur = DURATION_MAX as i32;
    } else if dur == 0 {
        dur = DURATION_MAX as i32;
    }

    // TODO: hardcoded constant
    let speed = target_velocity.abs().max() / 50;
    let old_dir = data.anim_dir(old_anim);
    let new_anim =
        if old_dir.is_none() && end_pos == start_pos {
            // Old anim was a special (non-physics) animation
            old_anim
        } else {
            let old_dir = old_dir.unwrap_or(0);
            let new_dir = velocity_dir(target_velocity, old_dir as usize);
            data.physics_anim_table()[speed as usize][new_dir as usize]
        };

    Motion {
        start_time: start_time,
        end_time: start_time + dur as Time,
        start_pos: start_pos,
        end_pos: end_pos,
        anim_id: new_anim,
    }
}

fn velocity_dir(v: V3, old_dir: usize) -> usize {
    let s = v.signum();
    let idx = (3 * (s.x + 1) + (s.y + 1)) as usize;
    [5, 4, 3, 6, old_dir, 2, 7, 0, 1][idx]
}
*/
