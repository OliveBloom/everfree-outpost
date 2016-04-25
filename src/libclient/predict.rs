use std::prelude::v1::*;
use std::collections::VecDeque;
use std::u16;

use physics::v3::{V3, Vn, scalar};
use physics::{self, ShapeSource};

use Time;
use data::Data;
use entity::Motion;


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
    /// The most recent movement direction, that is, the direction from the most recently processed
    /// input (as of the latest visible time).  We guess which inputs have been processed by
    /// comparing input timestamps to the start times of server-reported motions.
    last_dir: V3,
    /// Inputs whose timestamps are less than the future time, but for which we have not yet seen a
    /// response.
    inputs: VecDeque<Input>,

    /// The predicted movement direction as of the future time.
    cur_dir: V3,
    /// The predicted entity's current motion as of the future time.
    motion: Motion,

    /// Flag to indicate that the input log needs replaying.
    stale: bool,
}

impl Predictor {
    pub fn new() -> Predictor {
        Predictor {
            last_dir: scalar(0),
            inputs: VecDeque::new(),

            cur_dir: scalar(0),
            motion: Motion {
                start_pos: scalar(0),
                end_pos: scalar(0),
                start_time: 0,
                end_time: 1,
                anim_id: 0,
            },

            stale: false,
        }
    }

    pub fn canonical_motion(&mut self, motion: Motion) {
        self.motion = motion;

        // Pop all inputs that were handled by the server before it sent this motion.
        while self.inputs.len() > 0 {
            if self.inputs.front().unwrap().time > self.motion.start_time {
                break;
            }
            self.last_dir = self.inputs.pop_front().unwrap().dir;
        }

        self.stale = true;
    }

    pub fn input<S>(&mut self, time: Time, dir: V3, shape: &S, data: &Data)
            where S: ShapeSource {
        let input = Input { time: time, dir: dir };
        play_input(&mut self.motion,
                   &mut self.cur_dir,
                   &input,
                   shape,
                   data);
        self.inputs.push_back(input);
    }

    pub fn update<S>(&mut self, now: Time, shape: &S, data: &Data)
            where S: ShapeSource {
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
    }

    pub fn motion(&self) -> &Motion {
        &self.motion
    }
}

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
