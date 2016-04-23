use std::prelude::v1::*;
use std::collections::VecDeque;

use physics::v3::{V3, scalar};

use Time;
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

    pub fn canonical_motion<F>(&mut self, motion: Motion, predict: F)
            where F: Fn(V3, V3, Time) -> Motion {
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

    pub fn input<F>(&mut self, time: Time, dir: V3, predict: F)
            where F: Fn(V3, V3, Time) -> Motion {
        let input = Input { time: time, dir: dir };
        play_input(&mut self.motion,
                   &mut self.cur_dir,
                   &input,
                   predict);
        self.inputs.push_back(input);
    }

    pub fn update<F: Fn(V3, V3, Time) -> Motion>(&mut self, now: Time, predict: F) {
        if self.stale {
            // Replay all inputs.
            self.cur_dir = self.last_dir;
            for input in self.inputs.iter() {
                play_input(&mut self.motion,
                           &mut self.cur_dir,
                           input,
                           |pos, dir, time| predict(pos, dir, time));
            }
            self.stale = false;
        }

        while self.motion.end_time < now {
            self.motion = predict(self.motion.end_pos,
                                  self.cur_dir,
                                  self.motion.end_time);
        }
    }

    pub fn pos(&self, now: Time) -> V3 {
        self.motion.pos(now)
    }
}

fn play_input<F>(motion: &mut Motion,
                 dir: &mut V3,
                 input: &Input,
                 predict: F)
        where F: Fn(V3, V3, Time) -> Motion {
    // Play forward until the time of the input event.
    while motion.end_time < input.time {
        *motion = predict(motion.end_pos,
                          *dir,
                          motion.end_time);
    }

    // Play the input event.
    *dir = input.dir;
    *motion = predict(motion.pos(input.time),
                      *dir,
                      input.time);
}
