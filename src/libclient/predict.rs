use std::prelude::v1::*;
use std::collections::VecDeque;
use std::u16;

use physics::v3::{V3, Vn, scalar, Region};
use physics::ShapeSource;
use physics::walk2::Collider;

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

    /// Last canonical target velocity, derived from `pending_inputs` when we receive an "inputs
    /// processed" message.
    canon_target: V3,


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
            canon_target: scalar(0),

            current_motion: Motion::new(),
            // Initialize to a time far in the future.  The first `canonical_motion_update` will
            // rewind it to something reasonable.
            current_time: 65536,
        }
    }

    pub fn canonical_motion_update(&mut self, update: Update) {
        self.canon_motion.apply(update);
        self.current_motion = self.canon_motion.clone();
        // Rewind the current time to the time of the update, so that inputs will be replayed.
        self.current_time = update.when();
    }

    pub fn processed_inputs(&mut self, time: Time, count: usize) {
        for _ in 0 .. count {
            if self.pending_inputs.len() == 0 {
                warn!("ran out of pending inputs");
                break;
            }

            let i = self.pending_inputs.pop_front().unwrap();
            self.canon_target = i.dir;
        }

        self.current_time = time;
    }

    pub fn input<S>(&mut self, time: Time, dir: V3, shape: &S, data: &Data)
            where S: ShapeSource {
        let input = Input { time: time, dir: dir };
        self.pending_inputs.push_back(input);
    }

    pub fn update<S>(&mut self, future: Time, shape: &S, data: &Data)
            where S: ShapeSource {
        if future - self.current_time > TICK_MS * 3 {
            warn!("large time delta: {} .. {} ({} ticks)",
                  self.current_time, future, (future - self.current_time) / TICK_MS);
        }

        let cur_tick = future & !(TICK_MS - 1);
        let mut input_iter = self.pending_inputs.iter().peekable();
        let mut target_velocity = self.canon_target;
        while self.current_time < future {
            // This mimics the event loop body in server/logic/tick.rs
            let now = self.current_time;

            // Run through pending inputs to find the current target velocity.
            while input_iter.peek().map_or(false, |i| i.time <= now) {
                let i = input_iter.next().unwrap();
                target_velocity = i.dir;
            }

            // Take a step
            step_physics(shape, data, &mut self.current_motion, now, target_velocity);
            self.current_time += TICK_MS;
        }
    }

    pub fn motion(&self) -> &Motion {
        &self.current_motion
    }
}

fn step_physics<S>(shape: &S,
                   data: &Data,
                   motion: &mut Motion,
                   now: Time,
                   target_velocity: V3)
        where S: ShapeSource {
    let next = now + TICK_MS;

    let pos = motion.pos(now);
    let size = V3::new(32, 32, 48);
    let mut collider = Collider::new(shape, Region::new(pos, pos + size));

    let new_anim = walk_anim(data, motion.anim_id, target_velocity);

    // 1) Compute the actual velocity for this tick
    let velocity = collider.calc_velocity(target_velocity);
    let started = velocity != motion.velocity || new_anim != motion.anim_id;
    if started {
        *motion = Motion {
            start_pos: pos,
            velocity: velocity,
            start_time: now,
            end_time: None,
            anim_id: new_anim,
        };
    }

    let next_pos = motion.pos(next);
    let (step, dur) = collider.walk(next_pos - pos, TICK_MS as i32);
    let ended = dur != TICK_MS as i32;
    if ended {
        motion.end_time = Some(now + dur as Time);
    }
}

fn walk_anim(data: &Data, old_anim: u16, velocity: V3) -> u16 {
    let speed = velocity.abs().max() / 50;
    let dir_vec = velocity.signum();
    let idx = (3 * (dir_vec.x + 1) + (dir_vec.y + 1)) as usize;
    let old_dir = data.anim_dir(old_anim).unwrap_or(0);
    let dir = [5, 4, 3, 6, old_dir, 2, 7, 0, 1][idx];

    data.physics_anim_table()[speed as usize][dir as usize]
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
