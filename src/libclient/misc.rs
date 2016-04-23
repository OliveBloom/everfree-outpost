use std::prelude::v1::*;

use Time;
use data::Data;
use platform::{Config, ConfigKey};

/// Miscellaneous client state
pub struct Misc {
    pub hotbar: Hotbar,
    pub day_night: DayNight,
}

impl Misc {
    pub fn new() -> Misc {
        Misc {
            hotbar: Hotbar::new(),
            day_night: DayNight::new(),
        }
    }
}


#[derive(Clone, Copy)]
struct HotbarSlot {
    item_id: u16,
    is_ability: bool,
}

impl HotbarSlot {
    fn is_item(&self) -> bool {
        self.item_id != 0 && !self.is_ability
    }

    fn is_ability(&self) -> bool {
        self.item_id != 0 && self.is_ability
    }

    fn is_empty(&self) -> bool {
        self.item_id == 0
    }
}

pub struct Hotbar {
    slots: [HotbarSlot; 9],
    cur_item: i8,
    cur_ability: i8,
}

impl Hotbar {
    pub fn new() -> Hotbar {
        Hotbar {
            slots: [HotbarSlot { item_id: 0, is_ability: false }; 9],
            cur_item: -1,
            cur_ability: -1,
        }
    }

    pub fn init<C: Config>(&mut self, cfg: &C, data: &Data) {
        for i in 0 .. 9 {
            let name = cfg.get_str(ConfigKey::HotbarItemName(i as u8));
            let item_id = data.find_item_id(&name).unwrap_or(0);
            let is_item = cfg.get_int(ConfigKey::HotbarIsItem(i as u8)) != 0;

            self.slots[i].item_id = item_id;
            self.slots[i].is_ability = !is_item;
        }

        let idx = cfg.get_int(ConfigKey::HotbarActiveItem);
        if idx >= 0 && idx < 9 && self.slots[idx as usize].is_item() {
            self.cur_item = idx as i8;
        }
        // Note that a -1 in the config will fail the above test, and the value of self.cur_item
        // will stay at its default, -1.

        let idx = cfg.get_int(ConfigKey::HotbarActiveAbility);
        if idx >= 0 && idx < 9 && self.slots[idx as usize].is_ability() {
            self.cur_ability = idx as i8;
        }
    }

    pub fn item_id(&self, idx: u8) -> u16 {
        self.slots[idx as usize].item_id
    }

    pub fn is_item(&self, idx: u8) -> bool {
        self.slots[idx as usize].is_item()
    }

    pub fn is_ability(&self, idx: u8) -> bool {
        self.slots[idx as usize].is_ability()
    }

    pub fn active_item_index(&self) -> Option<u8> {
        if self.cur_item >= 0 && self.cur_item < 9 {
            Some(self.cur_item as u8)
        } else {
            None
        }
    }

    pub fn active_ability_index(&self) -> Option<u8> {
        if self.cur_ability >= 0 && self.cur_ability < 9 {
            Some(self.cur_ability as u8)
        } else {
            None
        }
    }

    pub fn active_item(&self) -> Option<u16> {
        if self.cur_item >= 0 && self.cur_item < 9 {
            let slot = &self.slots[self.cur_item as usize];
            if slot.is_item() {
                return Some(slot.item_id);
            }
        }
        None
    }

    pub fn active_ability(&self) -> Option<u16> {
        if self.cur_ability >= 0 && self.cur_ability < 9 {
            let slot = &self.slots[self.cur_ability as usize];
            if slot.is_ability() {
                return Some(slot.item_id);
            }
        }
        None
    }

    pub fn set_slot<C: Config>(&mut self,
                               data: &Data,
                               cfg: &mut C,
                               idx: u8, 
                               item_id: u16,
                               is_ability: bool) {
        if idx >= 9 {
            return;
        }

        self.slots[idx as usize].item_id = item_id;
        self.slots[idx as usize].is_ability = is_ability;

        let name = data.item_def(item_id).name();
        cfg.set_str(ConfigKey::HotbarItemName(idx), name);
        cfg.set_int(ConfigKey::HotbarIsItem(idx), (!is_ability) as i32);

        // Ensure cur_item and cur_ability are still valid
        if self.cur_item == idx as i8 && !self.slots[idx as usize].is_item() {
            self.cur_item = -1;
            cfg.set_int(ConfigKey::HotbarActiveItem, -1);
        }

        if self.cur_ability == idx as i8 && !self.slots[idx as usize].is_ability() {
            self.cur_ability = -1;
            cfg.set_int(ConfigKey::HotbarActiveAbility, -1);
        }
    }

    pub fn select<C: Config>(&mut self,
                             cfg: &mut C,
                             idx: u8) {
        if idx >= 9 {
            return;
        }

        let slot = &self.slots[idx as usize];
        if slot.is_item() {
            self.cur_item = idx as i8;
            cfg.set_int(ConfigKey::HotbarActiveItem, idx as i32);
        } else if slot.is_ability() {
            self.cur_ability = idx as i8;
            cfg.set_int(ConfigKey::HotbarActiveAbility, idx as i32);
        }
        // Otherwise, it's an empty slot, so do nothing.
    }

}


pub const DAY_NIGHT_CYCLE: u16 = 24000;

pub struct DayNight {
    base_time: Time,
    cycle_ms: Time,
    factor: Time,
}

impl DayNight {
    pub fn new() -> DayNight {
        DayNight {
            base_time: 0,
            cycle_ms: 24000,
            factor: 1,
        }
    }

    pub fn init(&mut self, base_time: Time, cycle_ms: Time) {
        self.base_time = base_time;
        self.cycle_ms = cycle_ms;
        self.factor = cycle_ms / DAY_NIGHT_CYCLE as i32;
    }

    /// Returns the time of day for the given timestamp.  Time of day is always in the range
    /// 0 .. DAY_NIGHT_CYCLE.
    pub fn time_of_day(&self, now: Time) -> u16 {
        let delta = now - self.base_time;
        let ms_time = delta % self.cycle_ms;
        let ms_time = if ms_time < 0 { ms_time + self.cycle_ms } else { ms_time };
        // The formula we want is:
        //    ms_time * DAY_NIGHT_CYCLE / self.cycle_ms
        // But that may require 64-bit arithmetic.  So instead we use this alternative formula,
        // which is just as good as long as cycle_ms is a multiple of DAY_NIGHT_CYCLE.
        (ms_time / self.factor) as u16
    }

    pub fn phase_delta(&self, data: &Data, time_of_day: u16) -> (u8, u16) {
        for (i, p) in data.day_night_phases().iter().enumerate() {
            if time_of_day < p.end_time {
                let start_time =
                    if i > 0 {
                        data.day_night_phases()[i - 1].end_time
                    } else {
                        0
                    };
                return (i as u8, time_of_day - start_time);
            }
        }
        panic!("no day/night phase covers time {}", time_of_day);
    }
}
