use std::prelude::v1::*;
use std::collections::VecDeque;

use physics::v3::V3;
use physics::ShapeSource;
use common_movement::{self, InputBits, INPUT_DIR_MASK, MovingEntity};

use Time;
use data::Data;
use entity::{Motion, Update};


struct Input {
    time: Time,
    input: InputBits,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    Walk,
    // Fly,
    Emote,  // Busy, interruptible
    Work,   // Busy, not interruptible
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

    /// Last canonical input, derived from `pending_inputs` when we receive an "inputs processed"
    /// message.
    canon_input: InputBits,

    /// Current activity, as reported by the server.
    canon_activity: Activity,


    /// The current motion as of the future time.
    current_motion: Motion,

    /// Timestamp of the last update.
    current_time: Time,

    me: MovingEntity,
}

// This must match the server tick.
pub const TICK_MS: i32 = 32;

impl Predictor {
    pub fn new() -> Predictor {
        Predictor {
            pending_inputs: VecDeque::new(),

            canon_motion: Motion::new(),
            canon_input: InputBits::empty(),
            canon_activity: Activity::Walk,

            current_motion: Motion::new(),
            // Initialize to a time far in the future.  The first `canonical_motion_update` will
            // rewind it to something reasonable.
            current_time: 65536,

            me: MovingEntity::new(),
        }
    }

    fn rewind(&mut self, when: Time) {
        self.current_time = when;
        self.me = MovingEntity::new();
        self.me.set_input(self.canon_input);
    }

    pub fn canonical_motion_update(&mut self, update: Update) {
        self.canon_motion.apply(update);
        self.current_motion = self.canon_motion.clone();
        // Rewind the current time to the time of the update, so that inputs will be replayed.
        self.rewind(update.when());
    }

    pub fn activity_update(&mut self, activity: Activity) {
        self.canon_activity = activity;
    }

    pub fn processed_inputs(&mut self, time: Time, count: usize) {
        for _ in 0 .. count {
            if self.pending_inputs.len() == 0 {
                warn!("ran out of pending inputs");
                break;
            }

            let i = self.pending_inputs.pop_front().unwrap();
            self.canon_input = i.input;
        }

        self.rewind(time);
    }

    pub fn input(&mut self, time: Time, input: InputBits) {
        let i = Input { time: time, input: input };
        self.pending_inputs.push_back(i);
    }

    pub fn update<S>(&mut self, future: Time, shape: &S, data: &Data)
            where S: ShapeSource {
        if future - self.current_time > TICK_MS * 3 {
            warn!("large time delta: {} .. {} ({} ticks)",
                  self.current_time, future, (future - self.current_time) / TICK_MS);
        }

        let mut input_iter = self.pending_inputs.iter().peekable();
        let mut activity = self.canon_activity;
        while self.current_time < future {
            // This mimics the event loop body in server/logic/tick.rs
            let now = self.current_time;

            // Run through pending inputs to find the current input bits.
            while input_iter.peek().map_or(false, |i| i.time <= now) {
                // `self.me` keeps track of the latest input internally.  Avoid replaying old
                // inputs, since that will force extra MovingEntity updates that didn't happen on
                // the server.
                let i = input_iter.next().unwrap();
                if i.time > now - TICK_MS {
                    self.me.set_input(i.input);
                }

                // Input will interrupt any active emote.
                if i.input & INPUT_DIR_MASK != InputBits::empty() &&
                   activity == Activity::Emote {
                    activity = Activity::Walk;
                }
            }

            // Take a step
            if activity == Activity::Walk {
                let mut e = EntityWrapper {
                    data: data,
                    motion: &mut self.current_motion,
                    activity: activity,
                };
                self.me.update(&mut e, shape, now, now + TICK_MS);
            }
            self.current_time += TICK_MS;
        }
    }

    pub fn motion(&self) -> &Motion {
        &self.current_motion
    }
}

struct EntityWrapper<'a, 'd> {
    data: &'d Data,
    motion: &'a mut Motion,
    activity: Activity,
}

impl<'a, 'd> common_movement::Entity for EntityWrapper<'a, 'd> {
    fn activity(&self) -> common_movement::Activity {
        match self.activity {
            Activity::Walk => common_movement::Activity::Walk,
            //Activity::Fly => common_movement::Activity::Fly,
            Activity::Emote |
            Activity::Work => common_movement::Activity::Busy,
        }
    }

    fn facing(&self) -> V3 {
        let dir = self.data.anim_dir(self.motion.anim_id).unwrap_or(0);
        let (x, y) = [
            ( 1,  0),
            ( 1,  1),
            ( 0,  1),
            (-1,  1),
            (-1,  0),
            (-1, -1),
            ( 0, -1),
            ( 1, -1),
        ][dir as usize];
        V3::new(x, y, 0)
    }

    fn velocity(&self) -> V3 {
        self.motion.velocity
    }


    type Time = Time;

    fn pos(&self, now: Time) -> V3 {
        self.motion.pos(now)
    }


    fn start_motion(&mut self, now: Time, pos: V3, facing: V3, speed: u8, velocity: V3) {
        *self.motion = Motion {
            start_pos: pos,
            velocity: velocity,
            start_time: now,
            end_time: None,
            anim_id: walk_anim(self.data, facing, speed),
        };
        println!("set velocity to {:?}", velocity);
    }

    fn end_motion(&mut self, now: Time) {
        self.motion.end_time = Some(now);
    }
}

fn walk_anim(data: &Data, facing: V3, speed: u8) -> u16 {
    let idx = (3 * (facing.x + 1) + (facing.y + 1)) as usize;
    let dir = [5, 4, 3, 6, 0, 2, 7, 0, 1][idx];

    data.physics_anim_table()[speed as usize][dir as usize]
}
